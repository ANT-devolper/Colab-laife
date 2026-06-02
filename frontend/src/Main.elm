module Main exposing (main)

{-| ColabLife SPA entry point. Two states: the sign-in page, and the authenticated
shell. The shell composes the cadastro write pages: `Page.Sectors`, `Page.Roles`
and `Page.Collaborators`.
-}

import Browser
import Html exposing (Html, div, h1, text)
import Html.Attributes exposing (class)
import Page.Collaborators as Collaborators
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
    , collaborators : Collaborators.Model
    }


type Msg
    = LoginMsg Login.Msg
    | SectorsMsg Sectors.Msg
    | RolesMsg Roles.Msg
    | CollaboratorsMsg Collaborators.Msg


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

                        ( collaborators, collaboratorsCmd ) =
                            Collaborators.init token
                    in
                    ( { model
                        | page =
                            AuthedView
                                { sectors = sectors
                                , roles = roles
                                , collaborators = collaborators
                                }
                      }
                    , Cmd.batch
                        [ Cmd.map SectorsMsg sectorsCmd
                        , Cmd.map RolesMsg rolesCmd
                        , Cmd.map CollaboratorsMsg collaboratorsCmd
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

        ( CollaboratorsMsg subMsg, AuthedView authed ) ->
            let
                ( collaborators, cmd ) =
                    Collaborators.update subMsg authed.collaborators
            in
            ( { model | page = AuthedView { authed | collaborators = collaborators } }
            , Cmd.map CollaboratorsMsg cmd
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
                , Html.map CollaboratorsMsg (Collaborators.view authed.collaborators)
                ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
