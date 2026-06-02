module Api exposing
    ( Credentials, LoginResponse
    , encodeCredentials, loginResponseDecoder
    , authHeader, login
    , Sector, Role, Collaborator
    , sectorDecoder, roleDecoder, collaboratorDecoder
    , getSectors, getRoles, getCollaborators
    , SectorForm, encodeSectorForm
    , createSector, updateSector, deleteSector
    , RoleForm, emptyRoleForm, roleFormFromRole, encodeRoleForm
    , createRole, updateRole, deleteRole
    , CollaboratorForm, emptyCollaboratorForm, collaboratorFormFromCollaborator, encodeCollaboratorForm
    , createCollaborator, updateCollaborator, deleteCollaborator
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
@docs RoleForm, emptyRoleForm, roleFormFromRole, encodeRoleForm
@docs createRole, updateRole, deleteRole
@docs CollaboratorForm, emptyCollaboratorForm, collaboratorFormFromCollaborator, encodeCollaboratorForm
@docs createCollaborator, updateCollaborator, deleteCollaborator

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


{-| A job title with the legacy set of optional description fields. They are
decoded as `Maybe` so an edit form can pre-fill from the current values.
-}
type alias Role =
    { id : String
    , name : String
    , profileSuggestion : Maybe String
    , objective : Maybe String
    , requirementEducation : Maybe String
    , requirementExperience : Maybe String
    , requirementAttention : Maybe String
    , requirementKnowledge : Maybe String
    , requirementSkill : Maybe String
    , requirementAttitude : Maybe String
    , requirementDelivery : Maybe String
    , observation : Maybe String
    , active : Bool
    }


{-| A person managed inside the tenant. The references and contact fields are
optional in the backend, so they are decoded as `Maybe` (an edit form pre-fills
from them).
-}
type alias Collaborator =
    { id : String
    , name : String
    , sectorId : Maybe String
    , roleId : Maybe String
    , managerId : Maybe String
    , whatsapp : Maybe String
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


{-| Decodes a single role, including its optional description fields.
-}
roleDecoder : Decoder Role
roleDecoder =
    Decode.succeed Role
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "name" Decode.string)
        |> andMap (optionalString "profile_suggestion")
        |> andMap (optionalString "objective")
        |> andMap (optionalString "requirement_education")
        |> andMap (optionalString "requirement_experience")
        |> andMap (optionalString "requirement_attention")
        |> andMap (optionalString "requirement_knowledge")
        |> andMap (optionalString "requirement_skill")
        |> andMap (optionalString "requirement_attitude")
        |> andMap (optionalString "requirement_delivery")
        |> andMap (optionalString "observation")
        |> andMap (Decode.field "active" Decode.bool)


{-| Applies a decoded argument to a decoded function — lets us build records with
more fields than `Decode.mapN` covers, without an extra dependency.
-}
andMap : Decoder a -> Decoder (a -> b) -> Decoder b
andMap =
    Decode.map2 (|>)


{-| A nullable/absent string field, decoded as `Maybe String`.
-}
optionalString : String -> Decoder (Maybe String)
optionalString name =
    Decode.maybe (Decode.field name Decode.string)


{-| Decodes a single collaborator, including its optional references and contact
fields.
-}
collaboratorDecoder : Decoder Collaborator
collaboratorDecoder =
    Decode.succeed Collaborator
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "name" Decode.string)
        |> andMap (optionalString "sector_id")
        |> andMap (optionalString "role_id")
        |> andMap (optionalString "manager_id")
        |> andMap (optionalString "whatsapp")
        |> andMap (optionalString "email")
        |> andMap (Decode.field "is_manager" Decode.bool)


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


{-| The create/update payload for a role: `name` plus the legacy optional fields,
each carried as a plain `String` (an empty string means "omit / leave untouched").
-}
type alias RoleForm =
    { name : String
    , profileSuggestion : String
    , objective : String
    , requirementEducation : String
    , requirementExperience : String
    , requirementAttention : String
    , requirementKnowledge : String
    , requirementSkill : String
    , requirementAttitude : String
    , requirementDelivery : String
    , observation : String
    }


{-| A blank role form (the starting point for creating a role).
-}
emptyRoleForm : RoleForm
emptyRoleForm =
    { name = ""
    , profileSuggestion = ""
    , objective = ""
    , requirementEducation = ""
    , requirementExperience = ""
    , requirementAttention = ""
    , requirementKnowledge = ""
    , requirementSkill = ""
    , requirementAttitude = ""
    , requirementDelivery = ""
    , observation = ""
    }


{-| Pre-fills a role form from an existing role (for editing). Missing fields
become empty strings.
-}
roleFormFromRole : Role -> RoleForm
roleFormFromRole role =
    { name = role.name
    , profileSuggestion = Maybe.withDefault "" role.profileSuggestion
    , objective = Maybe.withDefault "" role.objective
    , requirementEducation = Maybe.withDefault "" role.requirementEducation
    , requirementExperience = Maybe.withDefault "" role.requirementExperience
    , requirementAttention = Maybe.withDefault "" role.requirementAttention
    , requirementKnowledge = Maybe.withDefault "" role.requirementKnowledge
    , requirementSkill = Maybe.withDefault "" role.requirementSkill
    , requirementAttitude = Maybe.withDefault "" role.requirementAttitude
    , requirementDelivery = Maybe.withDefault "" role.requirementDelivery
    , observation = Maybe.withDefault "" role.observation
    }


{-| Encodes a role form: `name` is always present; each optional field is included
only when it is non-blank, so empty inputs are omitted from the JSON body.
-}
encodeRoleForm : RoleForm -> Encode.Value
encodeRoleForm form =
    Encode.object
        (( "name", Encode.string form.name )
            :: List.filterMap optionalPair
                [ ( "profile_suggestion", form.profileSuggestion )
                , ( "objective", form.objective )
                , ( "requirement_education", form.requirementEducation )
                , ( "requirement_experience", form.requirementExperience )
                , ( "requirement_attention", form.requirementAttention )
                , ( "requirement_knowledge", form.requirementKnowledge )
                , ( "requirement_skill", form.requirementSkill )
                , ( "requirement_attitude", form.requirementAttitude )
                , ( "requirement_delivery", form.requirementDelivery )
                , ( "observation", form.observation )
                ]
        )


{-| Turns a `(key, value)` pair into an encoded field, dropping blank values.
-}
optionalPair : ( String, String ) -> Maybe ( String, Encode.Value )
optionalPair ( key, rawValue ) =
    if String.trim rawValue == "" then
        Nothing

    else
        Just ( key, Encode.string rawValue )


{-| `POST /roles` — creates a role.
-}
createRole : String -> RoleForm -> (Result Http.Error Role -> msg) -> Cmd msg
createRole token form toMsg =
    authRequest token
        "POST"
        "/roles"
        (Http.jsonBody (encodeRoleForm form))
        (Http.expectJson toMsg roleDecoder)


{-| `PATCH /roles/{id}` — updates a role.
-}
updateRole : String -> String -> RoleForm -> (Result Http.Error Role -> msg) -> Cmd msg
updateRole token id form toMsg =
    authRequest token
        "PATCH"
        ("/roles/" ++ id)
        (Http.jsonBody (encodeRoleForm form))
        (Http.expectJson toMsg roleDecoder)


{-| `DELETE /roles/{id}` — deactivates a role (soft delete; backend replies `204`).
-}
deleteRole : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteRole token id toMsg =
    authRequest token "DELETE" ("/roles/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)


{-| The create/update payload for a collaborator. The references
(`sectorId`/`roleId`/`managerId`) and contact fields are plain `String`s where an
empty string means "none / leave untouched"; `isManager` is always sent.
-}
type alias CollaboratorForm =
    { name : String
    , sectorId : String
    , roleId : String
    , managerId : String
    , whatsapp : String
    , email : String
    , isManager : Bool
    }


{-| A blank collaborator form (the starting point for creating one).
-}
emptyCollaboratorForm : CollaboratorForm
emptyCollaboratorForm =
    { name = ""
    , sectorId = ""
    , roleId = ""
    , managerId = ""
    , whatsapp = ""
    , email = ""
    , isManager = False
    }


{-| Pre-fills a collaborator form from an existing collaborator (for editing).
Missing references/contacts become empty strings.
-}
collaboratorFormFromCollaborator : Collaborator -> CollaboratorForm
collaboratorFormFromCollaborator collaborator =
    { name = collaborator.name
    , sectorId = Maybe.withDefault "" collaborator.sectorId
    , roleId = Maybe.withDefault "" collaborator.roleId
    , managerId = Maybe.withDefault "" collaborator.managerId
    , whatsapp = Maybe.withDefault "" collaborator.whatsapp
    , email = Maybe.withDefault "" collaborator.email
    , isManager = collaborator.isManager
    }


{-| Encodes a collaborator form: `name` and `is_manager` are always present; each
reference/contact is included only when non-blank, so unset selects are omitted.
-}
encodeCollaboratorForm : CollaboratorForm -> Encode.Value
encodeCollaboratorForm form =
    Encode.object
        (( "name", Encode.string form.name )
            :: ( "is_manager", Encode.bool form.isManager )
            :: List.filterMap optionalPair
                [ ( "sector_id", form.sectorId )
                , ( "role_id", form.roleId )
                , ( "manager_id", form.managerId )
                , ( "whatsapp", form.whatsapp )
                , ( "email", form.email )
                ]
        )


{-| `POST /collaborators` — creates a collaborator.
-}
createCollaborator : String -> CollaboratorForm -> (Result Http.Error Collaborator -> msg) -> Cmd msg
createCollaborator token form toMsg =
    authRequest token
        "POST"
        "/collaborators"
        (Http.jsonBody (encodeCollaboratorForm form))
        (Http.expectJson toMsg collaboratorDecoder)


{-| `PATCH /collaborators/{id}` — updates a collaborator.
-}
updateCollaborator : String -> String -> CollaboratorForm -> (Result Http.Error Collaborator -> msg) -> Cmd msg
updateCollaborator token id form toMsg =
    authRequest token
        "PATCH"
        ("/collaborators/" ++ id)
        (Http.jsonBody (encodeCollaboratorForm form))
        (Http.expectJson toMsg collaboratorDecoder)


{-| `DELETE /collaborators/{id}` — deactivates a collaborator (soft delete; backend
replies `204`).
-}
deleteCollaborator : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteCollaborator token id toMsg =
    authRequest token "DELETE" ("/collaborators/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)
