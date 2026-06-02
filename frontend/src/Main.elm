module Main exposing (main)

{-| ColabLife SPA entry point. For this slice the app has two states: the sign-in
page, and a placeholder authenticated shell once a session token is obtained.
Read-only lists land in the next increment.
-}

import Browser
import Html exposing (Html)
import Page.Directory as Directory
import Page.Login as Login


type alias Model =
    { page : Page }


type Page
    = LoginView Login.Model
    | DirectoryView Directory.Model


type Msg
    = LoginMsg Login.Msg
    | DirectoryMsg Directory.Msg


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = LoginView Login.init }, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.page ) of
        ( LoginMsg subMsg, LoginView loginModel ) ->
            let
                ( newLoginModel, loginCmd, outMsg ) =
                    Login.update subMsg loginModel
            in
            case outMsg of
                Login.LoggedIn token ->
                    let
                        ( directoryModel, directoryCmd ) =
                            Directory.init token
                    in
                    ( { model | page = DirectoryView directoryModel }
                    , Cmd.map DirectoryMsg directoryCmd
                    )

                Login.NoOp ->
                    ( { model | page = LoginView newLoginModel }
                    , Cmd.map LoginMsg loginCmd
                    )

        ( DirectoryMsg subMsg, DirectoryView directoryModel ) ->
            let
                ( newDirectoryModel, directoryCmd ) =
                    Directory.update subMsg directoryModel
            in
            ( { model | page = DirectoryView newDirectoryModel }
            , Cmd.map DirectoryMsg directoryCmd
            )

        -- Messages that do not match the current page are ignored.
        ( LoginMsg _, DirectoryView _ ) ->
            ( model, Cmd.none )

        ( DirectoryMsg _, LoginView _ ) ->
            ( model, Cmd.none )


view : Model -> Browser.Document Msg
view model =
    { title = "ColabLife"
    , body = [ viewPage model.page ]
    }


viewPage : Page -> Html Msg
viewPage page =
    case page of
        LoginView loginModel ->
            Html.map LoginMsg (Login.view loginModel)

        DirectoryView directoryModel ->
            Html.map DirectoryMsg (Directory.view directoryModel)


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
