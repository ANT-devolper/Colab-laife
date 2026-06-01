module Page.Login exposing (Model, Msg, OutMsg(..), init, update, view)

{-| The sign-in page: an email/password form that exchanges credentials for a
session token. On success it reports the token to the caller through `OutMsg`,
keeping the page itself unaware of where the token is stored.

@docs Model, Msg, OutMsg, init, update, view

-}

import Api
import Html exposing (Html, button, div, form, h1, input, label, p, span, text)
import Html.Attributes exposing (class, disabled, type_, value)
import Html.Events exposing (onInput, onSubmit)
import Http


{-| Form state.
-}
type alias Model =
    { email : String
    , password : String
    , error : Maybe String
    , submitting : Bool
    }


{-| Internal messages.
-}
type Msg
    = EmailChanged String
    | PasswordChanged String
    | Submitted
    | GotResult (Result Http.Error Api.LoginResponse)


{-| Signal back to the parent: either nothing happened that it cares about, or
the user logged in and here is the session token.
-}
type OutMsg
    = NoOp
    | LoggedIn String


{-| A blank form.
-}
init : Model
init =
    { email = ""
    , password = ""
    , error = Nothing
    , submitting = False
    }


{-| Handles form edits and the login request lifecycle.
-}
update : Msg -> Model -> ( Model, Cmd Msg, OutMsg )
update msg model =
    case msg of
        EmailChanged email ->
            ( { model | email = email }, Cmd.none, NoOp )

        PasswordChanged password ->
            ( { model | password = password }, Cmd.none, NoOp )

        Submitted ->
            ( { model | submitting = True, error = Nothing }
            , Api.login { email = model.email, password = model.password } GotResult
            , NoOp
            )

        GotResult (Ok response) ->
            ( { model | submitting = False }, Cmd.none, LoggedIn response.token )

        GotResult (Err _) ->
            ( { model | submitting = False, error = Just "Invalid email or password." }
            , Cmd.none
            , NoOp
            )


{-| The sign-in form.
-}
view : Model -> Html Msg
view model =
    form [ class "login", onSubmit Submitted ]
        [ h1 [] [ text "Sign in" ]
        , label []
            [ span [] [ text "Email" ]
            , input
                [ type_ "email"
                , value model.email
                , onInput EmailChanged
                ]
                []
            ]
        , label []
            [ span [] [ text "Password" ]
            , input
                [ type_ "password"
                , value model.password
                , onInput PasswordChanged
                ]
                []
            ]
        , button [ type_ "submit", disabled model.submitting ]
            [ text
                (if model.submitting then
                    "Signing in…"

                 else
                    "Sign in"
                )
            ]
        , viewError model.error
        ]


viewError : Maybe String -> Html msg
viewError error =
    case error of
        Just message ->
            p [ class "error" ] [ text message ]

        Nothing ->
            text ""
