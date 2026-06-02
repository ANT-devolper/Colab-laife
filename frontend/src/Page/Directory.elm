module Page.Directory exposing (Model, Msg, init, update, view)

{-| Read-only tables of the tenant's collaborators and roles. (Sectors moved to
`Page.Sectors`, which adds write actions; collaborators and roles stay read-only
until their own write slices.) Each list is fetched on entry with the session
token; the view shows a loading, empty, error or populated state per list.

@docs Model, Msg, init, update, view

-}

import Api
import Html exposing (Html, div, em, h2, p, section, table, tbody, td, text, th, thead, tr)
import Html.Attributes exposing (class)
import Http


{-| The loading lifecycle of a fetched list.
-}
type Load a
    = Loading
    | Loaded a
    | Failed


{-| Page state: the token plus each list's load status.
-}
type alias Model =
    { token : String
    , collaborators : Load (List Api.Collaborator)
    , roles : Load (List Api.Role)
    }


{-| One message per list response.
-}
type Msg
    = GotCollaborators (Result Http.Error (List Api.Collaborator))
    | GotRoles (Result Http.Error (List Api.Role))


{-| Starts every list as `Loading` and fires the authenticated fetches.
-}
init : String -> ( Model, Cmd Msg )
init token =
    ( { token = token
      , collaborators = Loading
      , roles = Loading
      }
    , Cmd.batch
        [ Api.getCollaborators token GotCollaborators
        , Api.getRoles token GotRoles
        ]
    )


{-| Records each list's outcome.
-}
update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotCollaborators result ->
            ( { model | collaborators = fromResult result }, Cmd.none )

        GotRoles result ->
            ( { model | roles = fromResult result }, Cmd.none )


fromResult : Result Http.Error a -> Load a
fromResult result =
    case result of
        Ok value ->
            Loaded value

        Err _ ->
            Failed


{-| The two read-only sections.
-}
view : Model -> Html Msg
view model =
    div [ class "directory" ]
        [ viewList "Collaborators"
            "No collaborators yet."
            [ "Name", "Email" ]
            collaboratorRow
            model.collaborators
        , viewList "Roles"
            "No roles yet."
            [ "Name" ]
            namedRow
            (mapLoad (List.map .name) model.roles)
        ]


{-| Renders one section: a heading and either a status line or a table.
-}
viewList : String -> String -> List String -> (a -> Html msg) -> Load (List a) -> Html msg
viewList title emptyMessage headers rowView load =
    section [ class "directory-section" ]
        [ h2 [] [ text title ]
        , case load of
            Loading ->
                p [ class "status" ] [ text "Loading…" ]

            Failed ->
                p [ class "status error" ] [ text "Could not load this list." ]

            Loaded [] ->
                p [ class "status empty" ] [ em [] [ text emptyMessage ] ]

            Loaded items ->
                table []
                    [ thead [] [ tr [] (List.map (\h -> th [] [ text h ]) headers) ]
                    , tbody [] (List.map rowView items)
                    ]
        ]


collaboratorRow : Api.Collaborator -> Html msg
collaboratorRow collaborator =
    tr []
        [ td [] [ text collaborator.name ]
        , td [] [ text (Maybe.withDefault "—" collaborator.email) ]
        ]


namedRow : String -> Html msg
namedRow name =
    tr [] [ td [] [ text name ] ]


mapLoad : (a -> b) -> Load a -> Load b
mapLoad f load =
    case load of
        Loading ->
            Loading

        Failed ->
            Failed

        Loaded value ->
            Loaded (f value)
