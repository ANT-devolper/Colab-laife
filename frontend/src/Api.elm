module Api exposing
    ( Credentials, LoginResponse
    , encodeCredentials, loginResponseDecoder
    , authHeader, login
    , Sector, Role, Collaborator
    , sectorDecoder, roleDecoder, collaboratorDecoder
    , getSectors, getRoles, getCollaborators
    , SectorForm, encodeSectorForm
    , createSector, updateSector, deleteSector
    )

{-| HTTP boundary to the ColabLife backend.

The SPA is served from the same origin as the API (see ADR 0011), so every URL
here is a root-relative path — no base URL or CORS to deal with.

@docs Credentials, LoginResponse
@docs encodeCredentials, loginResponseDecoder
@docs authHeader, login
@docs Sector, Role, Collaborator
@docs sectorDecoder, roleDecoder, collaboratorDecoder
@docs getSectors, getRoles, getCollaborators
@docs SectorForm, encodeSectorForm
@docs createSector, updateSector, deleteSector

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



-- CADASTRO (read-only)


{-| An organizational unit.
-}
type alias Sector =
    { id : String
    , name : String
    , active : Bool
    }


{-| A job title. Only the fields the read-only list needs are decoded; the
backend carries more.
-}
type alias Role =
    { id : String
    , name : String
    , active : Bool
    }


{-| A person managed inside the tenant. `email` is optional in the backend, so it
is decoded as a `Maybe`.
-}
type alias Collaborator =
    { id : String
    , name : String
    , email : Maybe String
    , isManager : Bool
    }


{-| Decodes a single sector.
-}
sectorDecoder : Decoder Sector
sectorDecoder =
    Decode.map3 Sector
        (Decode.field "id" Decode.string)
        (Decode.field "name" Decode.string)
        (Decode.field "active" Decode.bool)


{-| Decodes a single role (ignoring the description fields the list does not show).
-}
roleDecoder : Decoder Role
roleDecoder =
    Decode.map3 Role
        (Decode.field "id" Decode.string)
        (Decode.field "name" Decode.string)
        (Decode.field "active" Decode.bool)


{-| Decodes a single collaborator.
-}
collaboratorDecoder : Decoder Collaborator
collaboratorDecoder =
    Decode.map4 Collaborator
        (Decode.field "id" Decode.string)
        (Decode.field "name" Decode.string)
        (Decode.field "email" (Decode.nullable Decode.string))
        (Decode.field "is_manager" Decode.bool)


{-| `GET /sectors` with the session token.
-}
getSectors : String -> (Result Http.Error (List Sector) -> msg) -> Cmd msg
getSectors token toMsg =
    authGet token "/sectors" (Decode.list sectorDecoder) toMsg


{-| `GET /roles` with the session token.
-}
getRoles : String -> (Result Http.Error (List Role) -> msg) -> Cmd msg
getRoles token toMsg =
    authGet token "/roles" (Decode.list roleDecoder) toMsg


{-| `GET /collaborators` with the session token.
-}
getCollaborators : String -> (Result Http.Error (List Collaborator) -> msg) -> Cmd msg
getCollaborators token toMsg =
    authGet token "/collaborators" (Decode.list collaboratorDecoder) toMsg


{-| A `GET` carrying the `Authorization: Bearer` header.
-}
authGet : String -> String -> Decoder a -> (Result Http.Error a -> msg) -> Cmd msg
authGet token url decoder toMsg =
    authRequest token "GET" url Http.emptyBody (Http.expectJson toMsg decoder)


{-| An authenticated request (any method) carrying the `Authorization: Bearer`
header. The caller chooses the body and how to interpret the response.
-}
authRequest : String -> String -> String -> Http.Body -> Http.Expect msg -> Cmd msg
authRequest token method url body expect =
    Http.request
        { method = method
        , headers = [ authHeader token ]
        , url = url
        , body = body
        , expect = expect
        , timeout = Nothing
        , tracker = Nothing
        }



-- CADASTRO (write)


{-| The create/update payload for a sector.
-}
type alias SectorForm =
    { name : String }


{-| Encodes a sector form into the JSON body the endpoint expects.
-}
encodeSectorForm : SectorForm -> Encode.Value
encodeSectorForm form =
    Encode.object [ ( "name", Encode.string form.name ) ]


{-| `POST /sectors` — creates a sector.
-}
createSector : String -> SectorForm -> (Result Http.Error Sector -> msg) -> Cmd msg
createSector token form toMsg =
    authRequest token
        "POST"
        "/sectors"
        (Http.jsonBody (encodeSectorForm form))
        (Http.expectJson toMsg sectorDecoder)


{-| `PATCH /sectors/{id}` — updates a sector.
-}
updateSector : String -> String -> SectorForm -> (Result Http.Error Sector -> msg) -> Cmd msg
updateSector token id form toMsg =
    authRequest token
        "PATCH"
        ("/sectors/" ++ id)
        (Http.jsonBody (encodeSectorForm form))
        (Http.expectJson toMsg sectorDecoder)


{-| `DELETE /sectors/{id}` — deactivates a sector (soft delete; backend replies
`204`, so there is no body to decode).
-}
deleteSector : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteSector token id toMsg =
    authRequest token "DELETE" ("/sectors/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)
