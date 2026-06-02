module Page.Annotations exposing (Model, Msg, init, update, view)

{-| Annotation management. Annotations are quick scored notes per collaborator, so
the page starts with a collaborator dropdown; picking one lists that collaborator's
annotations and binds the create/edit form to them. A single form serves create and
edit; the second score's amount-of-days input is shown only when "ask amount of
days" is checked. The list is re-fetched after each successful mutation.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, em, form, h2, input, label, option, p, section, select, span, table, tbody, td, text, textarea, th, thead, tr)
import Html.Attributes exposing (attribute, checked, class, disabled, selected, type_, value)
import Html.Events exposing (onCheck, onClick, onInput, onSubmit)
import Http


type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state. `selected` is the chosen collaborator id (`""` = none); `editing`
is `Just annotationId` while editing.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    , selected : String
    , annotations : Load (List Api.Annotation)
    , form : Api.AnnotationForm
    , editing : Maybe String
    , error : Maybe String
    }


type Msg
    = GotCollaborators (Result Http.Error (List Api.Collaborator))
    | CollaboratorSelected String
    | GotAnnotations (Result Http.Error (List Api.Annotation))
    | FieldChanged AField String
    | AskAmountToggled Bool
    | Submitted
    | Saved (Result Http.Error Api.Annotation)
    | EditClicked Api.Annotation
    | EditCancelled
    | DeactivateClicked String
    | Deactivated (Result Http.Error ())


type AField
    = NoteDate
    | Score1Number
    | Score1Type
    | Score1Description
    | AmountDays
    | Score2Number
    | Score2Type
    | Score2Description
    | MainNote
    | Observation


{-| Starts loading the collaborators (the annotation list waits for a selection).
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , collaborators = Loading
      , selected = ""
      , annotations = Loaded []
      , form = Api.emptyAnnotationForm ""
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
                    , annotations = Loaded []
                    , form = Api.emptyAnnotationForm ""
                    , editing = Nothing
                  }
                , Cmd.none
                )

            else
                ( { model
                    | selected = id
                    , annotations = Loading
                    , form = Api.emptyAnnotationForm id
                    , editing = Nothing
                    , error = Nothing
                  }
                , Api.getAnnotations model.token (Just id) GotAnnotations
                )

        GotAnnotations result ->
            ( { model | annotations = fromResult result }, Cmd.none )

        FieldChanged field fieldValue ->
            ( { model | form = setField field fieldValue model.form }, Cmd.none )

        AskAmountToggled flag ->
            ( { model | form = setAskAmount flag model.form }, Cmd.none )

        Submitted ->
            if model.selected == "" || String.isEmpty (String.trim model.form.noteDate) then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , case model.editing of
                    Just id ->
                        Api.updateAnnotation model.token id model.form Saved

                    Nothing ->
                        Api.createAnnotation model.token model.form Saved
                )

        Saved (Ok _) ->
            ( { model | form = Api.emptyAnnotationForm model.selected, editing = Nothing }
            , Api.getAnnotations model.token (Just model.selected) GotAnnotations
            )

        Saved (Err _) ->
            ( { model | error = Just "Could not save the annotation." }, Cmd.none )

        EditClicked annotation ->
            ( { model
                | editing = Just annotation.id
                , form = Api.annotationFormFromAnnotation annotation
                , error = Nothing
              }
            , Cmd.none
            )

        EditCancelled ->
            ( { model | editing = Nothing, form = Api.emptyAnnotationForm model.selected, error = Nothing }
            , Cmd.none
            )

        DeactivateClicked id ->
            ( { model | error = Nothing }, Api.deleteAnnotation model.token id Deactivated )

        Deactivated (Ok ()) ->
            ( model, Api.getAnnotations model.token (Just model.selected) GotAnnotations )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the annotation." }, Cmd.none )


setField : AField -> String -> Api.AnnotationForm -> Api.AnnotationForm
setField field fieldValue form =
    case field of
        NoteDate ->
            { form | noteDate = fieldValue }

        Score1Number ->
            { form | score1Number = Maybe.withDefault 0 (String.toInt fieldValue) }

        Score1Type ->
            { form | score1Type = fieldValue }

        Score1Description ->
            { form | score1Description = fieldValue }

        AmountDays ->
            { form | amountDays = fieldValue }

        Score2Number ->
            { form | score2Number = fieldValue }

        Score2Type ->
            { form | score2Type = fieldValue }

        Score2Description ->
            { form | score2Description = fieldValue }

        MainNote ->
            { form | mainNote = fieldValue }

        Observation ->
            { form | observation = fieldValue }


setAskAmount : Bool -> Api.AnnotationForm -> Api.AnnotationForm
setAskAmount flag form =
    { form | askAmountDays = flag }


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
        [ h2 [] [ text "Annotations" ]
        , viewCollaboratorPicker model
        , if model.selected == "" then
            p [ class "status empty" ] [ em [] [ text "Select a collaborator to manage annotations." ] ]

          else
            section []
                [ viewForm model
                , viewError model.error
                , viewList model.annotations
                ]
        ]


viewCollaboratorPicker : Model -> Html Msg
viewCollaboratorPicker model =
    label []
        [ span [] [ text "Collaborator" ]
        , select
            [ attribute "aria-label" "Annotation collaborator", onInput CollaboratorSelected ]
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
                "Save annotation"

            else
                "Create annotation"
    in
    form [ class "create-form", onSubmit Submitted ]
        ([ dateField "Note date" NoteDate model.form.noteDate
         , numberField "Score 1 number" Score1Number (String.fromInt model.form.score1Number)
         , textField "Score 1 type" Score1Type model.form.score1Type
         , textField "Score 1 description" Score1Description model.form.score1Description
         , label []
            [ input [ type_ "checkbox", checked model.form.askAmountDays, onCheck AskAmountToggled ] []
            , span [] [ text "Ask amount of days" ]
            ]
         ]
            ++ (if model.form.askAmountDays then
                    [ numberField "Amount of days" AmountDays model.form.amountDays ]

                else
                    []
               )
            ++ [ numberField "Score 2 number" Score2Number model.form.score2Number
               , textField "Score 2 type" Score2Type model.form.score2Type
               , textField "Score 2 description" Score2Description model.form.score2Description
               , longField "Main note" MainNote model.form.mainNote
               , longField "Annotation observation" Observation model.form.observation
               , button [ type_ "submit", disabled (String.isEmpty (String.trim model.form.noteDate)) ]
                    [ text submitLabel ]
               , if editing then
                    button [ type_ "button", onClick EditCancelled ] [ text "Cancel" ]

                 else
                    text ""
               ]
        )


dateField : String -> AField -> String -> Html Msg
dateField labelText field fieldValue =
    fieldWith "date" labelText field fieldValue


numberField : String -> AField -> String -> Html Msg
numberField labelText field fieldValue =
    fieldWith "number" labelText field fieldValue


textField : String -> AField -> String -> Html Msg
textField labelText field fieldValue =
    fieldWith "text" labelText field fieldValue


fieldWith : String -> String -> AField -> String -> Html Msg
fieldWith inputType labelText field fieldValue =
    label []
        [ span [] [ text labelText ]
        , input
            [ type_ inputType
            , attribute "aria-label" labelText
            , value fieldValue
            , onInput (FieldChanged field)
            ]
            []
        ]


longField : String -> AField -> String -> Html Msg
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


viewList : Load (List Api.Annotation) -> Html Msg
viewList load =
    case load of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load this list." ]

        Loaded [] ->
            p [ class "status empty" ] [ em [] [ text "No annotations yet." ] ]

        Loaded annotations ->
            table []
                [ thead []
                    [ tr []
                        [ th [] [ text "Date" ]
                        , th [] [ text "Score 1" ]
                        , th [] [ text "Type" ]
                        , th [] [ text "Actions" ]
                        ]
                    ]
                , tbody [] (List.map viewRow annotations)
                ]


viewRow : Api.Annotation -> Html Msg
viewRow annotation =
    tr []
        [ td [] [ text (String.left 10 annotation.noteDate) ]
        , td [] [ text (String.fromInt annotation.score1Number) ]
        , td [] [ text annotation.score1Type ]
        , td []
            [ button [ type_ "button", onClick (EditClicked annotation) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked annotation.id) ] [ text "Deactivate" ]
            ]
        ]
