module Page.Directory exposing (Model, Msg, init, update, view)

{-| Read-only table of the tenant's collaborators. (Sectors and roles moved to
`Page.Sectors`/`Page.Roles`, which add write actions; collaborators stay
read-only until their own write slice.) The list is fetched on entry with the
session token; the view shows a loading, empty, error or populated state.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, em, h2, p, section, table, tbody, td, text, th, thead, tr)
import Html.Attributes exposing (class)
import Http


{-| The loading lifecycle of a fetched list.
-}
type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state: the token plus the collaborators' load status.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    }


{-| The collaborators response.
-}
type Msg
    = GotCollaborators (Result Http.Error (List Api.Collaborator))


{-| Starts loading and fires the authenticated fetch.
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token, collaborators = Loading }
    , Api.getCollaborators token GotCollaborators
    )


{-| Records the list's outcome.
-}
update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotCollaborators result ->
            ( { model | collaborators = fromResult result }, Cmd.none )


fromResult : Result Http.Error a -> Load a
fromResult result =
    case result of
        Ok value ->
            Loaded value

        Err _ ->
            Failed


{-| The read-only collaborators section.
-}
view : Model -> Html Msg
view model =
    section [ class "directory-section" ]
        [ h2 [] [ text "Collaborators" ]
        , case model.collaborators of
            Loading ->
                p [ class "status" ] [ text "Loading…" ]

            Failed ->
                p [ class "status error" ] [ text "Could not load this list." ]

            Loaded [] ->
                p [ class "status empty" ] [ em [] [ text "No collaborators yet." ] ]

            Loaded collaborators ->
                table []
                    [ thead [] [ tr [] [ th [] [ text "Name" ], th [] [ text "Email" ] ] ]
                    , tbody [] (List.map collaboratorRow collaborators)
                    ]
        ]


collaboratorRow : Api.Collaborator -> Html msg
collaboratorRow collaborator =
    tr []
        [ td [] [ text collaborator.name ]
        , td [] [ text (Maybe.withDefault "—" collaborator.email) ]
        ]
