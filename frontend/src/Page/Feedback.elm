module Page.Feedback exposing (Model, Msg, init, update, view)

{-| Feedback management. Feedback is per collaborator, so the page starts with a
collaborator dropdown (from `/collaborators`); picking one lists that
collaborator's feedbacks and binds the create/edit form to them. A single form
serves create and edit (its mode is the `editing` id); the list is re-fetched
after each successful mutation.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, div, em, form, h2, input, label, li, option, p, section, select, span, table, tbody, td, text, textarea, th, thead, tr, ul)
import Html.Attributes exposing (attribute, checked, class, disabled, selected, type_, value)
import Html.Events exposing (onCheck, onClick, onInput, onSubmit)
import Http


type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state. `selected` is the chosen collaborator id (`""` = none); `editing`
is `Just feedbackId` while editing. `form` is bound to the selected collaborator.
`openFeedback` is the feedback whose expectation contract is being managed, with
its `items` and the two "new item" inputs.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    , selected : String
    , feedbacks : Load (List Api.Feedback)
    , form : Api.FeedbackForm
    , editing : Maybe String
    , error : Maybe String
    , openFeedback : Maybe String
    , items : Load (List Api.ExpectationItem)
    , newGoal : String
    , newBehavior : String
    , behaviors : Load (List Api.FeedbackBehavior)
    , behaviorForm : Api.FeedbackBehaviorForm
    , editingBehavior : Maybe String
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
    | OpenClicked String
    | CloseClicked
    | GotItems (Result Http.Error (List Api.ExpectationItem))
    | NewItemChanged ItemKind String
    | AddItem ItemKind
    | ToggleItem Api.ExpectationItem
    | RemoveItem String
    | ItemSaved (Result Http.Error Api.ExpectationItem)
    | ItemDeactivated (Result Http.Error ())
    | GotBehaviors (Result Http.Error (List Api.FeedbackBehavior))
    | BehaviorFieldChanged BField String
    | BehaviorSubmitted
    | BehaviorSaved (Result Http.Error Api.FeedbackBehavior)
    | BehaviorEditClicked Api.FeedbackBehavior
    | BehaviorEditCancelled
    | BehaviorRemoveClicked String
    | BehaviorRemoved (Result Http.Error ())


type Field
    = FeedbackDate
    | NextFeedbackDate
    | Status
    | Observation
    | ObservationPrivate


{-| The scored-behavior form's fields.
-}
type BField
    = BValue
    | BBehavior
    | BObs
    | BInstruction
    | BScore


{-| The two kinds of expectation-contract item.
-}
type ItemKind
    = Goal
    | Behavior


kindValue : ItemKind -> String
kindValue kind =
    case kind of
        Goal ->
            "goal"

        Behavior ->
            "behavior"


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
      , openFeedback = Nothing
      , items = Loaded []
      , newGoal = ""
      , newBehavior = ""
      , behaviors = Loaded []
      , behaviorForm = Api.emptyFeedbackBehaviorForm ""
      , editingBehavior = Nothing
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
                    , openFeedback = Nothing
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
            ( { model
                | openFeedback = Nothing
                , items = Loaded []
                , behaviors = Loaded []
                , editingBehavior = Nothing
              }
            , Api.getFeedbacks model.token (Just model.selected) GotFeedbacks
            )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the feedback." }, Cmd.none )

        OpenClicked id ->
            ( { model
                | openFeedback = Just id
                , items = Loading
                , newGoal = ""
                , newBehavior = ""
                , behaviors = Loading
                , behaviorForm = Api.emptyFeedbackBehaviorForm id
                , editingBehavior = Nothing
              }
            , Cmd.batch
                [ Api.getExpectationItems model.token id GotItems
                , Api.getFeedbackBehaviors model.token id GotBehaviors
                ]
            )

        CloseClicked ->
            ( { model
                | openFeedback = Nothing
                , items = Loaded []
                , behaviors = Loaded []
                , editingBehavior = Nothing
              }
            , Cmd.none
            )

        GotItems result ->
            ( { model | items = fromResult result }, Cmd.none )

        NewItemChanged Goal value ->
            ( { model | newGoal = value }, Cmd.none )

        NewItemChanged Behavior value ->
            ( { model | newBehavior = value }, Cmd.none )

        AddItem kind ->
            case ( model.openFeedback, newItemText kind model ) of
                ( Just feedbackId, description ) ->
                    if String.isEmpty (String.trim description) then
                        ( model, Cmd.none )

                    else
                        ( clearNewItem kind model
                        , Api.createExpectationItem model.token
                            { feedbackId = feedbackId
                            , kind = kindValue kind
                            , description = description
                            , done = False
                            }
                            ItemSaved
                        )

                ( Nothing, _ ) ->
                    ( model, Cmd.none )

        ToggleItem item ->
            ( model
            , Api.updateExpectationItem model.token
                item.id
                { feedbackId = item.feedbackId
                , kind = item.kind
                , description = Maybe.withDefault "" item.description
                , done = not item.done
                }
                ItemSaved
            )

        RemoveItem id ->
            ( model, Api.deleteExpectationItem model.token id ItemDeactivated )

        ItemSaved _ ->
            ( model, reloadItems model )

        ItemDeactivated _ ->
            ( model, reloadItems model )

        GotBehaviors result ->
            ( { model | behaviors = fromResult result }, Cmd.none )

        BehaviorFieldChanged field value ->
            ( { model | behaviorForm = setBField field value model.behaviorForm }, Cmd.none )

        BehaviorSubmitted ->
            if behaviorIncomplete model.behaviorForm then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , case model.editingBehavior of
                    Just id ->
                        Api.updateFeedbackBehavior model.token id model.behaviorForm BehaviorSaved

                    Nothing ->
                        Api.createFeedbackBehavior model.token model.behaviorForm BehaviorSaved
                )

        BehaviorSaved (Ok _) ->
            ( { model
                | behaviorForm = Api.emptyFeedbackBehaviorForm (openFeedbackId model)
                , editingBehavior = Nothing
              }
            , reloadBehaviors model
            )

        BehaviorSaved (Err _) ->
            ( { model | error = Just "Could not save the behavior." }, Cmd.none )

        BehaviorEditClicked behavior ->
            ( { model
                | editingBehavior = Just behavior.id
                , behaviorForm = Api.feedbackBehaviorFormFromBehavior behavior
                , error = Nothing
              }
            , Cmd.none
            )

        BehaviorEditCancelled ->
            ( { model
                | editingBehavior = Nothing
                , behaviorForm = Api.emptyFeedbackBehaviorForm (openFeedbackId model)
              }
            , Cmd.none
            )

        BehaviorRemoveClicked id ->
            ( model, Api.deleteFeedbackBehavior model.token id BehaviorRemoved )

        BehaviorRemoved _ ->
            ( { model | editingBehavior = Nothing }, reloadBehaviors model )


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


newItemText : ItemKind -> Model -> String
newItemText kind model =
    case kind of
        Goal ->
            model.newGoal

        Behavior ->
            model.newBehavior


clearNewItem : ItemKind -> Model -> Model
clearNewItem kind model =
    case kind of
        Goal ->
            { model | newGoal = "" }

        Behavior ->
            { model | newBehavior = "" }


reloadItems : Model -> Cmd Msg
reloadItems model =
    case model.openFeedback of
        Just id ->
            Api.getExpectationItems model.token id GotItems

        Nothing ->
            Cmd.none


reloadBehaviors : Model -> Cmd Msg
reloadBehaviors model =
    case model.openFeedback of
        Just id ->
            Api.getFeedbackBehaviors model.token id GotBehaviors

        Nothing ->
            Cmd.none


openFeedbackId : Model -> String
openFeedbackId model =
    Maybe.withDefault "" model.openFeedback


setBField : BField -> String -> Api.FeedbackBehaviorForm -> Api.FeedbackBehaviorForm
setBField field value form =
    case field of
        BValue ->
            { form | valueDescription = value }

        BBehavior ->
            { form | behaviorDescription = value }

        BObs ->
            { form | behaviorObs = value }

        BInstruction ->
            { form | valueInstruction = value }

        BScore ->
            { form | score = Maybe.withDefault 0 (String.toInt value) }


behaviorIncomplete : Api.FeedbackBehaviorForm -> Bool
behaviorIncomplete form =
    String.isEmpty (String.trim form.valueDescription)
        || String.isEmpty (String.trim form.behaviorDescription)


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
                , viewList model.openFeedback model.feedbacks
                , viewContract model
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


viewList : Maybe String -> Load (List Api.Feedback) -> Html Msg
viewList openFeedback load =
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
                , tbody [] (List.map (viewRow openFeedback) feedbacks)
                ]


viewRow : Maybe String -> Api.Feedback -> Html Msg
viewRow openFeedback feedback =
    let
        openButton =
            if openFeedback == Just feedback.id then
                button [ type_ "button", onClick CloseClicked ] [ text "Close" ]

            else
                button [ type_ "button", onClick (OpenClicked feedback.id) ] [ text "Open" ]
    in
    tr []
        [ td [] [ text (String.left 10 feedback.feedbackDate) ]
        , td [] [ text (Maybe.withDefault "—" feedback.status) ]
        , td []
            [ openButton
            , button [ type_ "button", onClick (EditClicked feedback) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked feedback.id) ] [ text "Deactivate" ]
            ]
        ]


{-| The expectation contract of the open feedback: two checklists (goals,
behaviors). Hidden when no feedback is open.
-}
viewContract : Model -> Html Msg
viewContract model =
    case model.openFeedback of
        Nothing ->
            text ""

        Just _ ->
            section [ class "contract" ]
                [ h2 [] [ text "Expectation contract" ]
                , viewItems model
                , viewBehaviors model
                ]


viewItems : Model -> Html Msg
viewItems model =
    case model.items of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load the contract." ]

        Loaded loadedItems ->
            div []
                [ viewChecklist "Goals" Goal "New goal" "Add goal" model.newGoal loadedItems
                , viewChecklist "Behaviors" Behavior "New behavior" "Add behavior" model.newBehavior loadedItems
                ]


viewBehaviors : Model -> Html Msg
viewBehaviors model =
    section [ class "scored-behaviors" ]
        [ h2 [] [ text "Scored behaviors" ]
        , viewBehaviorForm model
        , case model.behaviors of
            Loading ->
                p [ class "status" ] [ text "Loading…" ]

            Failed ->
                p [ class "status error" ] [ text "Could not load the behaviors." ]

            Loaded [] ->
                p [ class "status empty" ] [ em [] [ text "No scored behaviors yet." ] ]

            Loaded behaviors ->
                table []
                    [ thead []
                        [ tr []
                            [ th [] [ text "Value" ]
                            , th [] [ text "Behavior" ]
                            , th [] [ text "Score" ]
                            , th [] [ text "Actions" ]
                            ]
                        ]
                    , tbody [] (List.map viewBehaviorRow behaviors)
                    ]
        ]


viewBehaviorForm : Model -> Html Msg
viewBehaviorForm model =
    let
        editing =
            model.editingBehavior /= Nothing

        submitLabel =
            if editing then
                "Save scored behavior"

            else
                "Add scored behavior"
    in
    form [ class "create-form", onSubmit BehaviorSubmitted ]
        [ bTextField "Value description" BValue model.behaviorForm.valueDescription
        , bTextField "Behavior description" BBehavior model.behaviorForm.behaviorDescription
        , bLongField "Behavior observation" BObs model.behaviorForm.behaviorObs
        , bLongField "Value instruction" BInstruction model.behaviorForm.valueInstruction
        , label []
            [ span [] [ text "Behavior score" ]
            , input
                [ type_ "number"
                , attribute "aria-label" "Behavior score"
                , value (String.fromInt model.behaviorForm.score)
                , onInput (BehaviorFieldChanged BScore)
                ]
                []
            ]
        , button [ type_ "submit", disabled (behaviorIncomplete model.behaviorForm) ]
            [ text submitLabel ]
        , if editing then
            button [ type_ "button", onClick BehaviorEditCancelled ] [ text "Cancel" ]

          else
            text ""
        ]


bTextField : String -> BField -> String -> Html Msg
bTextField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , input
            [ attribute "aria-label" labelText, value fieldValue, onInput (BehaviorFieldChanged field) ]
            []
        ]


bLongField : String -> BField -> String -> Html Msg
bLongField labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , textarea
            [ attribute "aria-label" labelText, value fieldValue, onInput (BehaviorFieldChanged field) ]
            []
        ]


viewBehaviorRow : Api.FeedbackBehavior -> Html Msg
viewBehaviorRow behavior =
    tr []
        [ td [] [ text behavior.valueDescription ]
        , td [] [ text behavior.behaviorDescription ]
        , td [] [ text (String.fromInt behavior.score) ]
        , td []
            [ button [ type_ "button", onClick (BehaviorEditClicked behavior) ] [ text "Edit" ]
            , button [ type_ "button", onClick (BehaviorRemoveClicked behavior.id) ] [ text "Remove" ]
            ]
        ]


viewChecklist : String -> ItemKind -> String -> String -> String -> List Api.ExpectationItem -> Html Msg
viewChecklist title kind inputLabel addLabel newValue items =
    let
        ofKind =
            List.filter (\item -> item.kind == kindValue kind) items
    in
    section [ class "checklist" ]
        [ h2 [] [ text title ]
        , ul [] (List.map viewItem ofKind)
        , form [ class "create-form", onSubmit (AddItem kind) ]
            [ label []
                [ span [] [ text inputLabel ]
                , input
                    [ attribute "aria-label" inputLabel
                    , value newValue
                    , onInput (NewItemChanged kind)
                    ]
                    []
                ]
            , button
                [ type_ "submit", disabled (String.isEmpty (String.trim newValue)) ]
                [ text addLabel ]
            ]
        ]


viewItem : Api.ExpectationItem -> Html Msg
viewItem item =
    let
        description =
            Maybe.withDefault "—" item.description
    in
    li []
        [ label []
            [ input
                [ type_ "checkbox"
                , checked item.done
                , attribute "aria-label" description
                , onCheck (\_ -> ToggleItem item)
                ]
                []
            , span [] [ text description ]
            ]
        , button [ type_ "button", onClick (RemoveItem item.id) ] [ text "Remove" ]
        ]
