module Main exposing (main)

{-| ColabLife SPA entry point. For this slice the app has two states: the sign-in
page, and a placeholder authenticated shell once a session token is obtained.
Read-only lists land in the next increment.
-}

import Browser
import Html exposing (Html, div, text)
import Html.Attributes exposing (class)
import Page.Login as Login


type alias Model =
    { page : Page }


type Page
    = LoginView Login.Model
    | Authenticated Session


type alias Session =
    { token : String }


type Msg
    = LoginMsg Login.Msg


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
                    ( { model | page = Authenticated { token = token } }, Cmd.none )

                Login.NoOp ->
                    ( { model | page = LoginView newLoginModel }
                    , Cmd.map LoginMsg loginCmd
                    )

        -- No login messages are expected once authenticated.
        ( LoginMsg _, Authenticated _ ) ->
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

        Authenticated _ ->
            div [ class "shell" ] [ text "Signed in." ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
