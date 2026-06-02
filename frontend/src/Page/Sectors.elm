module Page.Sectors exposing (Model, Msg, init, update, view)

{-| Sector management: the tenant's sectors with create, inline edit and
deactivate. The list is the source of truth — it is re-fetched after every
successful mutation. Requires the session token for its authenticated calls.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, button, em, form, h2, input, label, p, section, span, table, tbody, td, text, th, thead, tr)
import Html.Attributes exposing (attribute, class, disabled, type_, value)
import Html.Events exposing (onClick, onInput, onSubmit)
import Http


{-| The loading lifecycle of the fetched list.
-}
type Load a
    = Loading
    | Loaded a
    | Failed


{-| One row being edited: its id and the in-progress name.
-}
type alias Editing =
    { id : String, name : String }


{-| Page state.
-}
type alias Model =
    { token : String
    , sectors : Load (List Api.Sector)
    , newName : String
    , editing : Maybe Editing
    , error : Maybe String
    }


type Msg
    = Got (Result Http.Error (List Api.Sector))
    | NewNameChanged String
    | CreateSubmitted
    | Created (Result Http.Error Api.Sector)
    | EditClicked Api.Sector
    | EditNameChanged String
    | EditCancelled
    | EditSaved
    | Saved (Result Http.Error Api.Sector)
    | DeactivateClicked String
    | Deactivated (Result Http.Error ())


{-| Starts loading and fetches the sector list.
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , sectors = Loading
      , newName = ""
      , editing = Nothing
      , error = Nothing
      }
    , Api.getSectors token Got
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Got result ->
            ( { model | sectors = fromResult result }, Cmd.none )

        NewNameChanged name ->
            ( { model | newName = name }, Cmd.none )

        CreateSubmitted ->
            if String.isEmpty (String.trim model.newName) then
                ( model, Cmd.none )

            else
                ( { model | error = Nothing }
                , Api.createSector model.token { name = model.newName } Created
                )

        Created (Ok _) ->
            ( { model | newName = "" }, Api.getSectors model.token Got )

        Created (Err _) ->
            ( { model | error = Just "Could not create the sector." }, Cmd.none )

        EditClicked sector ->
            ( { model | editing = Just { id = sector.id, name = sector.name }, error = Nothing }
            , Cmd.none
            )

        EditNameChanged name ->
            ( { model | editing = Maybe.map (\e -> { e | name = name }) model.editing }
            , Cmd.none
            )

        EditCancelled ->
            ( { model | editing = Nothing }, Cmd.none )

        EditSaved ->
            case model.editing of
                Just editing ->
                    ( { model | error = Nothing }
                    , Api.updateSector model.token editing.id { name = editing.name } Saved
                    )

                Nothing ->
                    ( model, Cmd.none )

        Saved (Ok _) ->
            ( { model | editing = Nothing }, Api.getSectors model.token Got )

        Saved (Err _) ->
            ( { model | error = Just "Could not save the sector." }, Cmd.none )

        DeactivateClicked id ->
            ( { model | error = Nothing }, Api.deleteSector model.token id Deactivated )

        Deactivated (Ok ()) ->
            ( model, Api.getSectors model.token Got )

        Deactivated (Err _) ->
            ( { model | error = Just "Could not deactivate the sector." }, Cmd.none )


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
        [ h2 [] [ text "Sectors" ]
        , viewCreateForm model.newName
        , viewError model.error
        , viewList model.editing model.sectors
        ]


viewCreateForm : String -> Html Msg
viewCreateForm newName =
    form [ class "create-form", onSubmit CreateSubmitted ]
        [ label []
            [ span [] [ text "New sector" ]
            , input
                [ attribute "aria-label" "New sector name"
                , value newName
                , onInput NewNameChanged
                ]
                []
            ]
        , button
            [ type_ "submit", disabled (String.isEmpty (String.trim newName)) ]
            [ text "Create sector" ]
        ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just message ->
            p [ class "error" ] [ text message ]

        Nothing ->
            text ""


viewList : Maybe Editing -> Load (List Api.Sector) -> Html Msg
viewList editing load =
    case load of
        Loading ->
            p [ class "status" ] [ text "Loading…" ]

        Failed ->
            p [ class "status error" ] [ text "Could not load this list." ]

        Loaded [] ->
            p [ class "status empty" ] [ em [] [ text "No sectors yet." ] ]

        Loaded sectors ->
            table []
                [ thead [] [ tr [] [ th [] [ text "Name" ], th [] [ text "Actions" ] ] ]
                , tbody [] (List.map (viewRow editing) sectors)
                ]


viewRow : Maybe Editing -> Api.Sector -> Html Msg
viewRow editing sector =
    case editing of
        Just e ->
            if e.id == sector.id then
                viewEditRow e

            else
                viewReadRow sector

        Nothing ->
            viewReadRow sector


viewReadRow : Api.Sector -> Html Msg
viewReadRow sector =
    tr []
        [ td [] [ text sector.name ]
        , td []
            [ button [ type_ "button", onClick (EditClicked sector) ] [ text "Edit" ]
            , button [ type_ "button", onClick (DeactivateClicked sector.id) ] [ text "Deactivate" ]
            ]
        ]


viewEditRow : Editing -> Html Msg
viewEditRow editing =
    tr []
        [ td []
            [ input
                [ attribute "aria-label" "Edit sector name"
                , value editing.name
                , onInput EditNameChanged
                ]
                []
            ]
        , td []
            [ button [ type_ "button", onClick EditSaved ] [ text "Save" ]
            , button [ type_ "button", onClick EditCancelled ] [ text "Cancel" ]
            ]
        ]
