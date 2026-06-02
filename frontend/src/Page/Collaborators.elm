module Page.Collaborators exposing (Model, Msg, init, update, view)

{-| Collaborator management: the tenant's collaborators with create, edit and
deactivate. A single form serves both create and edit; its sector/role/manager
fields are dropdowns populated from the active sectors, roles and collaborators,
which the page fetches alongside the collaborator list. The list is the source of
truth and is re-fetched after every successful mutation; a failed save (e.g. the
backend's `422` for a dangling reference) surfaces as a form error.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, em, form, h2, input, label, option, p, section, select, span, table, tbody, td, text, th, thead, tr)
import Html.Attributes exposing (attribute, checked, class, disabled, selected, type_, value)
import Html.Events exposing (onCheck, onClick, onInput, onSubmit)
import Http


{-| The loading lifecycle of a fetched list.
-}
type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state. `editing` is `Just id` while editing an existing collaborator,
`Nothing` while creating one; `form` holds the in-progress values either way. The
sector/role lists feed the dropdowns.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    , sectors : Load (List Api.Sector)
    , roles : Load (List Api.Role)
    , form : Api.CollaboratorForm
    , editing : Maybe String
    , error : Maybe String
    }


type Msg
    = GotCollaborators (Result Http.Error (List Api.Collaborator))
    | GotSectors (Result Http.Error (List Api.Sector))
    | GotRoles (Result Http.Error (List Api.Role))
    | FieldChanged Field String
    | ManagerToggled Bool
    | Submitted
    | Saved (Result Http.Error Api.Collaborator)
    | EditClicked Api.Collaborator
    | EditCancelled
    | DeactivateClicked String
    | Deactivated (Result Http.Error ())


{-| The form's text/select fields (the manager flag has its own message).
-}
type Field
    = Name
    | SectorId
    | RoleId
    | ManagerId
    | Whatsapp
    | Email


{-| Starts loading and fetches the three lists the page needs.
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , collaborators = Loading
      , sectors = Loading
      , roles = Loading
      , form = Api.emptyCollaboratorForm
      , editing = Nothing
      , error = Nothing
      }
    , Cmd.batch
        [ Api.getCollaborators token GotCollaborators
        , Api.getSectors token GotSectors
        , Api.getRoles token GotRoles
        ]
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotCollaborators result ->
            ( { model | collaborators = fromResult result }, Cmd.none )

        GotSectors result ->
            ( { model | sectors = fromResult result }, Cmd.none )

        GotRoles result ->
            ( { model | roles = fromResult result }, Cmd.none )

        FieldChanged field fieldValue ->
            ( { model | form = setField field fieldValue model.form }, Cmd.none )

        ManagerToggled flag ->
            ( { model | form = setManager flag model.form }, Cmd.none )

        Submitted ->
            if String.isEmpty (String.trim model.form.name) then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , case model.editing of
                    Just id ->
                        Api.updateCollaborator model.token id model.form Saved

                    Nothing ->
                        Api.createCollaborator model.token model.form Saved
                )

        Saved (Ok _) ->
            ( { model | form = Api.emptyCollaboratorForm, editing = Nothing }
            , Api.getCollaborators model.token GotCollaborators
            )

        Saved (Err _) ->
            ( { model | error = Just "Could not save the collaborator. Check the sector, role and manager." }
            , Cmd.none
            )

        EditClicked collaborator ->
            ( { model
                | editing = Just collaborator.id
                , form = Api.collaboratorFormFromCollaborator collaborator
                , error = Nothing
              }
            , Cmd.none
            )

        EditCancelled ->
            ( { model | editing = Nothing, form = Api.emptyCollaboratorForm, error = Nothing }
            , Cmd.none
            )

        DeactivateClicked id ->
            ( { model | error = Nothing }, Api.deleteCollaborator model.token id Deactivated )

        Deactivated (Ok ()) ->
            ( model, Api.getCollaborators model.token GotCollaborators )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the collaborator." }, Cmd.none )


setField : Field -> String -> Api.CollaboratorForm -> Api.CollaboratorForm
setField field fieldValue form =
    case field of
        Name ->
            { form | name = fieldValue }

        SectorId ->
            { form | sectorId = fieldValue }

        RoleId ->
            { form | roleId = fieldValue }

        ManagerId ->
            { form | managerId = fieldValue }

        Whatsapp ->
            { form | whatsapp = fieldValue }

        Email ->
            { form | email = fieldValue }


setManager : Bool -> Api.CollaboratorForm -> Api.CollaboratorForm
setManager flag form =
    { form | isManager = flag }


fromResult : Result Http.Error a -> Load a
fromResult result =
    case result of
        Ok value ->
            Loaded value

        Err _ ->
            Failed


loadedOr : List a -> Load (List a) -> List a
loadedOr fallback load =
    case load of
        Loaded value ->
            value

        _ ->
            fallback


view : Model -> Html Msg
view model =
    section [ class "directory-section" ]
        [ h2 [] [ text "Collaborators" ]
        , viewForm model
        , viewError model.error
        , viewList model.collaborators
        ]


viewForm : Model -> Html Msg
viewForm model =
    let
        editing =
            model.editing /= Nothing

        submitLabel =
            if editing then
                "Save collaborator"

            else
                "Create collaborator"

        sectorOptions =
            loadedOr [] model.sectors |> List.map (\s -> ( s.id, s.name ))

        roleOptions =
            loadedOr [] model.roles |> List.map (\r -> ( r.id, r.name ))

        managerOptions =
            loadedOr [] model.collaborators
                |> List.filter (\c -> Just c.id /= model.editing)
                |> List.map (\c -> ( c.id, c.name ))
    in
    form [ class "create-form", onSubmit Submitted ]
        [ textField "Collaborator name" Name model.form.name
        , selectField "Collaborator sector" "— no sector —" SectorId model.form.sectorId sectorOptions
        , selectField "Collaborator role" "— no role —" RoleId model.form.roleId roleOptions
        , selectField "Collaborator manager" "— no manager —" ManagerId model.form.managerId managerOptions
        , textField "Collaborator WhatsApp" Whatsapp model.form.whatsapp
        , textField "Collaborator email" Email model.form.email
        , label []
            [ input
                [ type_ "checkbox", checked model.form.isManager, onCheck ManagerToggled ]
                []
            , span [] [ text "Is manager" ]
            ]
        , button
            [ type_ "submit", disabled (String.isEmpty (String.trim model.form.name)) ]
            [ text submitLabel ]
        , if editing then
            button [ type_ "button", onClick EditCancelled ] [ text "Cancel" ]

          else
            text ""
        ]


textField : String -> Field -> String -> Html Msg
textField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , input
            [ attribute "aria-label" labelText
            , value fieldValue
            , onInput (FieldChanged field)
            ]
            []
        ]


selectField : String -> String -> Field -> String -> List ( String, String ) -> Html Msg
selectField labelText noneLabel field current options =
    label []
        [ span [] [ text labelText ]
        , select
            [ attribute "aria-label" labelText, onInput (FieldChanged field) ]
            (option [ value "", selected (current == "") ] [ text noneLabel ]
                :: List.map
                    (\( id, name ) ->
                        option [ value id, selected (current == id) ] [ text name ]
                    )
                    options
            )
        ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just message ->
            p [ class "error" ] [ text message ]

        Nothing ->
            text ""


viewList : Load (List Api.Collaborator) -> Html Msg
viewList load =
    case load of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load this list." ]

        Loaded [] ->
            p [ class "status empty" ] [ em [] [ text "No collaborators yet." ] ]

        Loaded collaborators ->
            table []
                [ thead []
                    [ tr []
                        [ th [] [ text "Name" ], th [] [ text "Email" ], th [] [ text "Actions" ] ]
                    ]
                , tbody [] (List.map viewRow collaborators)
                ]


viewRow : Api.Collaborator -> Html Msg
viewRow collaborator =
    tr []
        [ td [] [ text collaborator.name ]
        , td [] [ text (Maybe.withDefault "—" collaborator.email) ]
        , td []
            [ button [ type_ "button", onClick (EditClicked collaborator) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked collaborator.id) ] [ text "Deactivate" ]
            ]
        ]
