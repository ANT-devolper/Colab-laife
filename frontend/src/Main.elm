module Main exposing (main)

{-| ColabLife SPA entry point. Two states: the sign-in page, and the authenticated
shell. The shell composes `Page.Sectors` and `Page.Roles` (full write CRUD) with
the still read-only `Page.Directory` (collaborators).
-}

import Browser
import Html exposing (Html, div, h1, text)
import Html.Attributes exposing (class)
import Page.Directory as Directory
import Page.Login as Login
import Page.Roles as Roles
import Page.Sectors as Sectors


type alias Model =
    { page : Page }


type Page
    = LoginView Login.Model
    | AuthedView Authed


{-| The authenticated shell's sub-pages.
-}
type alias Authed =
    { sectors : Sectors.Model
    , roles : Roles.Model
    , directory : Directory.Model
    }


type Msg
    = LoginMsg Login.Msg
    | SectorsMsg Sectors.Msg
    | RolesMsg Roles.Msg
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
                        ( sectors, sectorsCmd ) =
                            Sectors.init token

                        ( roles, rolesCmd ) =
                            Roles.init token

                        ( directory, directoryCmd ) =
                            Directory.init token
                    in
                    ( { model
                        | page =
                            AuthedView
                                { sectors = sectors
                                , roles = roles
                                , directory = directory
                                }
                      }
                    , Cmd.batch
                        [ Cmd.map SectorsMsg sectorsCmd
                        , Cmd.map RolesMsg rolesCmd
                        , Cmd.map DirectoryMsg directoryCmd
                        ]
                    )

                Login.NoOp ->
                    ( { model | page = LoginView newLoginModel }
                    , Cmd.map LoginMsg loginCmd
                    )

        ( SectorsMsg subMsg, AuthedView authed ) ->
            let
                ( sectors, cmd ) =
                    Sectors.update subMsg authed.sectors
            in
            ( { model | page = AuthedView { authed | sectors = sectors } }
            , Cmd.map SectorsMsg cmd
            )

        ( RolesMsg subMsg, AuthedView authed ) ->
            let
                ( roles, cmd ) =
                    Roles.update subMsg authed.roles
            in
            ( { model | page = AuthedView { authed | roles = roles } }
            , Cmd.map RolesMsg cmd
            )

        ( DirectoryMsg subMsg, AuthedView authed ) ->
            let
                ( directory, cmd ) =
                    Directory.update subMsg authed.directory
            in
            ( { model | page = AuthedView { authed | directory = directory } }
            , Cmd.map DirectoryMsg cmd
            )

        -- Messages that do not match the current page are ignored.
        _ ->
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

        AuthedView authed ->
            div [ class "directory" ]
                [ h1 [] [ text "Directory" ]
                , Html.map SectorsMsg (Sectors.view authed.sectors)
                , Html.map RolesMsg (Roles.view authed.roles)
                , Html.map DirectoryMsg (Directory.view authed.directory)
                ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
