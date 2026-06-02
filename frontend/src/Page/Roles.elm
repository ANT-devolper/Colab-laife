module Page.Roles exposing (Model, Msg, init, update, view)

{-| Role management: the tenant's roles with create, edit and deactivate. A single
form serves both create and edit (its mode is the `editing` id); the list is the
source of truth and is re-fetched after every successful mutation. Requires the
session token for its authenticated calls.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, em, form, h2, input, label, p, section, span, table, tbody, td, text, textarea, th, thead, tr)
import Html.Attributes exposing (attribute, class, disabled, type_, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Http


{-| The loading lifecycle of the fetched list.
-}
type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state. `editing` is `Just id` while editing an existing role, `Nothing`
while creating a new one; `form` holds the in-progress field values either way.
-}
type alias Model =
    { token : String
    , roles : Load (List Api.Role)
    , form : Api.RoleForm
    , editing : Maybe String
    , error : Maybe String
    }


type Msg
    = Got (Result Http.Error (List Api.Role))
    | FieldChanged Field String
    | Submitted
    | Saved (Result Http.Error Api.Role)
    | EditClicked Api.Role
    | EditCancelled
    | DeactivateClicked String
    | Deactivated (Result Http.Error ())


{-| The form's editable fields, so a single message carries every input change.
-}
type Field
    = Name
    | ProfileSuggestion
    | Objective
    | RequirementEducation
    | RequirementExperience
    | RequirementAttention
    | RequirementKnowledge
    | RequirementSkill
    | RequirementAttitude
    | RequirementDelivery
    | Observation


{-| Starts loading and fetches the role list.
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , roles = Loading
      , form = Api.emptyRoleForm
      , editing = Nothing
      , error = Nothing
      }
    , Api.getRoles token Got
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Got result ->
            ( { model | roles = fromResult result }, Cmd.none )

        FieldChanged field value ->
            ( { model | form = setField field value model.form }, Cmd.none )

        Submitted ->
            if String.isEmpty (String.trim model.form.name) then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , case model.editing of
                    Just id ->
                        Api.updateRole model.token id model.form Saved

                    Nothing ->
                        Api.createRole model.token model.form Saved
                )

        Saved (Ok _) ->
            ( { model | form = Api.emptyRoleForm, editing = Nothing }
            , Api.getRoles model.token Got
            )

        Saved (Err _) ->
            ( { model | error = Just "Could not save the role." }, Cmd.none )

        EditClicked role ->
            ( { model
                | editing = Just role.id
                , form = Api.roleFormFromRole role
                , error = Nothing
              }
            , Cmd.none
            )

        EditCancelled ->
            ( { model | editing = Nothing, form = Api.emptyRoleForm, error = Nothing }
            , Cmd.none
            )

        DeactivateClicked id ->
            ( { model | error = Nothing }, Api.deleteRole model.token id Deactivated )

        Deactivated (Ok ()) ->
            ( model, Api.getRoles model.token Got )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the role." }, Cmd.none )


setField : Field -> String -> Api.RoleForm -> Api.RoleForm
setField field value form =
    case field of
        Name ->
            { form | name = value }

        ProfileSuggestion ->
            { form | profileSuggestion = value }

        Objective ->
            { form | objective = value }

        RequirementEducation ->
            { form | requirementEducation = value }

        RequirementExperience ->
            { form | requirementExperience = value }

        RequirementAttention ->
            { form | requirementAttention = value }

        RequirementKnowledge ->
            { form | requirementKnowledge = value }

        RequirementSkill ->
            { form | requirementSkill = value }

        RequirementAttitude ->
            { form | requirementAttitude = value }

        RequirementDelivery ->
            { form | requirementDelivery = value }

        Observation ->
            { form | observation = value }


fromResult : Result Http.Error a -> Load a
fromResult result =
    case result of
        Ok value ->
            Loaded value

        Err _ ->
            Failed


view : Model -> Html Msg
view model =
    section [ class "directory-section" ]
        [ h2 [] [ text "Roles" ]
        , viewForm model
        , viewError model.error
        , viewList model.roles
        ]


viewForm : Model -> Html Msg
viewForm model =
    let
        editing =
            model.editing /= Nothing

        submitLabel =
            if editing then
                "Save role"

            else
                "Create role"
    in
    form [ class "create-form", onSubmit Submitted ]
        (textField "Role name" Name model.form.name
            :: List.map (\( lbl, field, val ) -> longField lbl field val)
                [ ( "Profile suggestion", ProfileSuggestion, model.form.profileSuggestion )
                , ( "Objective", Objective, model.form.objective )
                , ( "Requirement: education", RequirementEducation, model.form.requirementEducation )
                , ( "Requirement: experience", RequirementExperience, model.form.requirementExperience )
                , ( "Requirement: attention", RequirementAttention, model.form.requirementAttention )
                , ( "Requirement: knowledge", RequirementKnowledge, model.form.requirementKnowledge )
                , ( "Requirement: skill", RequirementSkill, model.form.requirementSkill )
                , ( "Requirement: attitude", RequirementAttitude, model.form.requirementAttitude )
                , ( "Requirement: delivery", RequirementDelivery, model.form.requirementDelivery )
                , ( "Observation", Observation, model.form.observation )
                ]
            ++ [ button
                    [ type_ "submit", disabled (String.isEmpty (String.trim model.form.name)) ]
                    [ text submitLabel ]
               , if editing then
                    button [ type_ "button", onClick EditCancelled ] [ text "Cancel" ]

                 else
                    text ""
               ]
        )


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


longField : String -> Field -> String -> Html Msg
longField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , textarea
            [ attribute "aria-label" labelText
            , value fieldValue
            , onInput (FieldChanged field)
            ]
            []
        ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just message ->
            p [ class "error" ] [ text message ]

        Nothing ->
            text ""


viewList : Load (List Api.Role) -> Html Msg
viewList load =
    case load of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load this list." ]

        Loaded [] ->
            p [ class "status empty" ] [ em [] [ text "No roles yet." ] ]

        Loaded roles ->
            table []
                [ thead [] [ tr [] [ th [] [ text "Name" ], th [] [ text "Actions" ] ] ]
                , tbody [] (List.map viewRow roles)
                ]


viewRow : Api.Role -> Html Msg
viewRow role =
    tr []
        [ td [] [ text role.name ]
        , td []
            [ button [ type_ "button", onClick (EditClicked role) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked role.id) ] [ text "Deactivate" ]
            ]
        ]
