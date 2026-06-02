module Main exposing (main)

{-| ColabLife SPA entry point. Two states: the sign-in page, and the authenticated
shell. The shell has tabs: "Cadastro" (sectors, roles and collaborators write CRUD)
and "Feedback" (per-collaborator feedback).
-}

import Browser
import Html exposing (Html, button, div, h1, nav, text)
import Html.Attributes exposing (class, classList, type_)
import Html.Events exposing (onClick)
import Page.Annotations as Annotations
import Page.Collaborators as Collaborators
import Page.Feedback as Feedback
import Page.Login as Login
import Page.Roles as Roles
import Page.Sectors as Sectors


type alias Model =
    { page : Page }


type Page
    = LoginView Login.Model
    | AuthedView Authed


{-| The active shell tab.
-}
type Tab
    = CadastroTab
    | FeedbackTab
    | AnnotationsTab


{-| The authenticated shell's tabs and sub-pages.
-}
type alias Authed =
    { tab : Tab
    , sectors : Sectors.Model
    , roles : Roles.Model
    , collaborators : Collaborators.Model
    , feedback : Feedback.Model
    , annotations : Annotations.Model
    }


type Msg
    = LoginMsg Login.Msg
    | TabSelected Tab
    | SectorsMsg Sectors.Msg
    | RolesMsg Roles.Msg
    | CollaboratorsMsg Collaborators.Msg
    | FeedbackMsg Feedback.Msg
    | AnnotationsMsg Annotations.Msg


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

                        ( feedback, feedbackCmd ) =
                            Feedback.init token

                        ( annotations, annotationsCmd ) =
                            Annotations.init token
                    in
                    ( { model
                        | page =
                            AuthedView
                                { tab = CadastroTab
                                , sectors = sectors
                                , roles = roles
                                , collaborators = collaborators
                                , feedback = feedback
                                , annotations = annotations
                                }
                      }
                    , Cmd.batch
                        [ Cmd.map SectorsMsg sectorsCmd
                        , Cmd.map RolesMsg rolesCmd
                        , Cmd.map CollaboratorsMsg collaboratorsCmd
                        , Cmd.map FeedbackMsg feedbackCmd
                        , Cmd.map AnnotationsMsg annotationsCmd
                        ]
                    )

                Login.NoOp ->
                    ( { model | page = LoginView newLoginModel }
                    , Cmd.map LoginMsg loginCmd
                    )

        ( TabSelected tab, AuthedView authed ) ->
            ( { model | page = AuthedView { authed | tab = tab } }, Cmd.none )

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

        ( FeedbackMsg subMsg, AuthedView authed ) ->
            let
                ( feedback, cmd ) =
                    Feedback.update subMsg authed.feedback
            in
            ( { model | page = AuthedView { authed | feedback = feedback } }
            , Cmd.map FeedbackMsg cmd
            )

        ( AnnotationsMsg subMsg, AuthedView authed ) ->
            let
                ( annotations, cmd ) =
                    Annotations.update subMsg authed.annotations
            in
            ( { model | page = AuthedView { authed | annotations = annotations } }
            , Cmd.map AnnotationsMsg cmd
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
            div [ class "app" ]
                [ h1 [] [ text "Directory" ]
                , viewTabs authed.tab
                , viewTab authed
                ]


viewTabs : Tab -> Html Msg
viewTabs active =
    nav [ class "tabs" ]
        [ tabButton CadastroTab "Cadastro" active
        , tabButton FeedbackTab "Feedback" active
        , tabButton AnnotationsTab "Annotations" active
        ]


tabButton : Tab -> String -> Tab -> Html Msg
tabButton tab label active =
    button
        [ type_ "button", classList [ ( "active", tab == active ) ], onClick (TabSelected tab) ]
        [ text label ]


viewTab : Authed -> Html Msg
viewTab authed =
    case authed.tab of
        CadastroTab ->
            div [ class "directory" ]
                [ Html.map SectorsMsg (Sectors.view authed.sectors)
                , Html.map RolesMsg (Roles.view authed.roles)
                , Html.map CollaboratorsMsg (Collaborators.view authed.collaborators)
                ]

        FeedbackTab ->
            div [ class "directory" ]
                [ Html.map FeedbackMsg (Feedback.view authed.feedback) ]

        AnnotationsTab ->
            div [ class "directory" ]
                [ Html.map AnnotationsMsg (Annotations.view authed.annotations) ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , update = update
        , view = view
        , subscriptions = \_ -> Sub.none
        }
