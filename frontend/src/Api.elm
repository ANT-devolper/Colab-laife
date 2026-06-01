module Api exposing
    ( Credentials, LoginResponse
    , encodeCredentials, loginResponseDecoder
    , authHeader, login
    )

{-| HTTP boundary to the ColabLife backend.

The SPA is served from the same origin as the API (see ADR 0011), so every URL
here is a root-relative path — no base URL or CORS to deal with.

@docs Credentials, LoginResponse
@docs encodeCredentials, loginResponseDecoder
@docs authHeader, login

-}

import Http
import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode


{-| The login form's payload.
-}
type alias Credentials =
    { email : String
    , password : String
    }


{-| The session token returned by `POST /auth/login`.
-}
type alias LoginResponse =
    { token : String
    , tokenType : String
    }


{-| Encodes credentials into the JSON body the login endpoint expects.
-}
encodeCredentials : Credentials -> Encode.Value
encodeCredentials credentials =
    Encode.object
        [ ( "email", Encode.string credentials.email )
        , ( "password", Encode.string credentials.password )
        ]


{-| Decodes the login response (`{ token, token_type }`).
-}
loginResponseDecoder : Decoder LoginResponse
loginResponseDecoder =
    Decode.map2 LoginResponse
        (Decode.field "token" Decode.string)
        (Decode.field "token_type" Decode.string)


{-| Builds the `Authorization: Bearer <token>` header for authenticated requests.
-}
authHeader : String -> Http.Header
authHeader token =
    Http.header "Authorization" ("Bearer " ++ token)


{-| Exchanges credentials for a session token via `POST /auth/login`.
-}
login : Credentials -> (Result Http.Error LoginResponse -> msg) -> Cmd msg
login credentials toMsg =
    Http.post
        { url = "/auth/login"
        , body = Http.jsonBody (encodeCredentials credentials)
        , expect = Http.expectJson toMsg loginResponseDecoder
        }
