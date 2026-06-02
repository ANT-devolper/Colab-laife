module Page.Feedback exposing (Model, Msg, init, update, view)

{-| Feedback management. Feedback is per collaborator, so the page starts with a
collaborator dropdown (from `/collaborators`); picking one lists that
collaborator's feedbacks and binds the create/edit form to them. A single form
serves create and edit (its mode is the `editing` id); the list is re-fetched
after each successful mutation.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, em, form, h2, input, label, option, p, section, select, span, table, tbody, td, text, textarea, th, thead, tr)
import Html.Attributes exposing (attribute, class, disabled, selected, type_, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Http


type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state. `selected` is the chosen collaborator id (`""` = none); `editing`
is `Just feedbackId` while editing. `form` is bound to the selected collaborator.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    , selected : String
    , feedbacks : Load (List Api.Feedback)
    , form : Api.FeedbackForm
    , editing : Maybe String
    , error : Maybe String
    }


type Msg
    = GotCollaborators (Result Http.Error (List Api.Collaborator))
    | CollaboratorSelected String
    | GotFeedbacks (Result Http.Error (List Api.Feedback))
    | FieldChanged Field String
    | Submitted
    | Saved (Result Http.Error Api.Feedback)
    | EditClicked Api.Feedback
    | EditCancelled
    | DeactivateClicked String
    | Deactivated (Result Http.Error ())


type Field
    = FeedbackDate
    | NextFeedbackDate
    | Status
    | Observation
    | ObservationPrivate


{-| Starts loading the collaborators (the feedback list waits for a selection).
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , collaborators = Loading
      , selected = ""
      , feedbacks = Loaded []
      , form = Api.emptyFeedbackForm ""
      , editing = Nothing
      , error = Nothing
      }
    , Api.getCollaborators token GotCollaborators
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotCollaborators result ->
            ( { model | collaborators = fromResult result }, Cmd.none )

        CollaboratorSelected id ->
            if id == "" then
                ( { model
                    | selected = ""
                    , feedbacks = Loaded []
                    , form = Api.emptyFeedbackForm ""
                    , editing = Nothing
                  }
                , Cmd.none
                )

            else
                ( { model
                    | selected = id
                    , feedbacks = Loading
                    , form = Api.emptyFeedbackForm id
                    , editing = Nothing
                    , error = Nothing
                  }
                , Api.getFeedbacks model.token (Just id) GotFeedbacks
                )

        GotFeedbacks result ->
            ( { model | feedbacks = fromResult result }, Cmd.none )

        FieldChanged field fieldValue ->
            ( { model | form = setField field fieldValue model.form }, Cmd.none )

        Submitted ->
            if model.selected == "" || String.isEmpty (String.trim model.form.feedbackDate) then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , case model.editing of
                    Just id ->
                        Api.updateFeedback model.token id model.form Saved

                    Nothing ->
                        Api.createFeedback model.token model.form Saved
                )

        Saved (Ok _) ->
            ( { model | form = Api.emptyFeedbackForm model.selected, editing = Nothing }
            , Api.getFeedbacks model.token (Just model.selected) GotFeedbacks
            )

        Saved (Err _) ->
            ( { model | error = Just "Could not save the feedback." }, Cmd.none )

        EditClicked feedback ->
            ( { model
                | editing = Just feedback.id
                , form = Api.feedbackFormFromFeedback feedback
                , error = Nothing
              }
            , Cmd.none
            )

        EditCancelled ->
            ( { model | editing = Nothing, form = Api.emptyFeedbackForm model.selected, error = Nothing }
            , Cmd.none
            )

        DeactivateClicked id ->
            ( { model | error = Nothing }, Api.deleteFeedback model.token id Deactivated )

        Deactivated (Ok ()) ->
            ( model, Api.getFeedbacks model.token (Just model.selected) GotFeedbacks )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the feedback." }, Cmd.none )


setField : Field -> String -> Api.FeedbackForm -> Api.FeedbackForm
setField field fieldValue form =
    case field of
        FeedbackDate ->
            { form | feedbackDate = fieldValue }

        NextFeedbackDate ->
            { form | nextFeedbackDate = fieldValue }

        Status ->
            { form | status = fieldValue }

        Observation ->
            { form | observation = fieldValue }

        ObservationPrivate ->
            { form | observationPrivate = fieldValue }


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
        [ h2 [] [ text "Feedback" ]
        , viewCollaboratorPicker model
        , if model.selected == "" then
            p [ class "status empty" ] [ em [] [ text "Select a collaborator to manage feedback." ] ]

          else
            section []
                [ viewForm model
                , viewError model.error
                , viewList model.feedbacks
                ]
        ]


viewCollaboratorPicker : Model -> Html Msg
viewCollaboratorPicker model =
    label []
        [ span [] [ text "Collaborator" ]
        , select
            [ attribute "aria-label" "Feedback collaborator", onInput CollaboratorSelected ]
            (option [ value "", selected (model.selected == "") ] [ text "— select —" ]
                :: List.map
                    (\c ->
                        option [ value c.id, selected (model.selected == c.id) ] [ text c.name ]
                    )
                    (loadedOr [] model.collaborators)
            )
        ]


viewForm : Model -> Html Msg
viewForm model =
    let
        editing =
            model.editing /= Nothing

        submitLabel =
            if editing then
                "Save feedback"

            else
                "Create feedback"
    in
    form [ class "create-form", onSubmit Submitted ]
        [ dateField "Feedback date" FeedbackDate model.form.feedbackDate
        , dateField "Next feedback date" NextFeedbackDate model.form.nextFeedbackDate
        , textField "Feedback status" Status model.form.status
        , longField "Feedback observation" Observation model.form.observation
        , longField "Feedback private observation" ObservationPrivate model.form.observationPrivate
        , button
            [ type_ "submit", disabled (String.isEmpty (String.trim model.form.feedbackDate)) ]
            [ text submitLabel ]
        , if editing then
            button [ type_ "button", onClick EditCancelled ] [ text "Cancel" ]

          else
            text ""
        ]


dateField : String -> Field -> String -> Html Msg
dateField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , input
            [ type_ "date"
            , attribute "aria-label" labelText
            , value fieldValue
            , onInput (FieldChanged field)
            ]
            []
        ]


textField : String -> Field -> String -> Html Msg
textField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , input
            [ attribute "aria-label" labelText, value fieldValue, onInput (FieldChanged field) ]
            []
        ]


longField : String -> Field -> String -> Html Msg
longField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , textarea
            [ attribute "aria-label" labelText, value fieldValue, onInput (FieldChanged field) ]
            []
        ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just message ->
            p [ class "error" ] [ text message ]

        Nothing ->
            text ""


viewList : Load (List Api.Feedback) -> Html Msg
viewList load =
    case load of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load this list." ]

        Loaded [] ->
            p [ class "status empty" ] [ em [] [ text "No feedback yet." ] ]

        Loaded feedbacks ->
            table []
                [ thead []
                    [ tr []
                        [ th [] [ text "Date" ], th [] [ text "Status" ], th [] [ text "Actions" ] ]
                    ]
                , tbody [] (List.map viewRow feedbacks)
                ]


viewRow : Api.Feedback -> Html Msg
viewRow feedback =
    tr []
        [ td [] [ text (String.left 10 feedback.feedbackDate) ]
        , td [] [ text (Maybe.withDefault "—" feedback.status) ]
        , td []
            [ button [ type_ "button", onClick (EditClicked feedback) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked feedback.id) ] [ text "Deactivate" ]
            ]
        ]
