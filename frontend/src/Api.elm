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
    , Feedback, feedbackDecoder, getFeedbacks
    , FeedbackForm, emptyFeedbackForm, feedbackFormFromFeedback, encodeFeedbackForm
    , createFeedback, updateFeedback, deleteFeedback
    , ExpectationItem, expectationItemDecoder, getExpectationItems
    , ExpectationItemForm, encodeExpectationItemForm
    , createExpectationItem, updateExpectationItem, deleteExpectationItem
    , FeedbackBehavior, feedbackBehaviorDecoder, getFeedbackBehaviors
    , FeedbackBehaviorForm, emptyFeedbackBehaviorForm, feedbackBehaviorFormFromBehavior, encodeFeedbackBehaviorForm
    , createFeedbackBehavior, updateFeedbackBehavior, deleteFeedbackBehavior
    , Annotation, annotationDecoder, getAnnotations
    , AnnotationForm, emptyAnnotationForm, annotationFormFromAnnotation, encodeAnnotationForm
    , createAnnotation, updateAnnotation, deleteAnnotation
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
@docs Feedback, feedbackDecoder, getFeedbacks
@docs FeedbackForm, emptyFeedbackForm, feedbackFormFromFeedback, encodeFeedbackForm
@docs createFeedback, updateFeedback, deleteFeedback
@docs ExpectationItem, expectationItemDecoder, getExpectationItems
@docs ExpectationItemForm, encodeExpectationItemForm
@docs createExpectationItem, updateExpectationItem, deleteExpectationItem
@docs FeedbackBehavior, feedbackBehaviorDecoder, getFeedbackBehaviors
@docs FeedbackBehaviorForm, emptyFeedbackBehaviorForm, feedbackBehaviorFormFromBehavior, encodeFeedbackBehaviorForm
@docs createFeedbackBehavior, updateFeedbackBehavior, deleteFeedbackBehavior
@docs Annotation, annotationDecoder, getAnnotations
@docs AnnotationForm, emptyAnnotationForm, annotationFormFromAnnotation, encodeAnnotationForm
@docs createAnnotation, updateAnnotation, deleteAnnotation

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



-- FEEDBACK


{-| A feedback for a collaborator. `feedbackDate`/`nextFeedbackDate` are the raw
RFC3339 strings from the backend (the UI shows the date part); the contract
observations and status are optional.
-}
type alias Feedback =
    { id : String
    , collaboratorId : String
    , feedbackDate : String
    , nextFeedbackDate : Maybe String
    , observation : Maybe String
    , observationPrivate : Maybe String
    , status : Maybe String
    , active : Bool
    }


{-| Decodes a single feedback.
-}
feedbackDecoder : Decoder Feedback
feedbackDecoder =
    Decode.succeed Feedback
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "collaborator_id" Decode.string)
        |> andMap (Decode.field "feedback_date" Decode.string)
        |> andMap (optionalString "next_feedback_date")
        |> andMap (optionalString "expectation_contract_observation")
        |> andMap (optionalString "expectation_contract_observation_private")
        |> andMap (optionalString "status")
        |> andMap (Decode.field "active" Decode.bool)


{-| `GET /feedbacks` with the session token; an optional collaborator id narrows
the list to one collaborator (the backend's `?collaborator_id=` filter).
-}
getFeedbacks : String -> Maybe String -> (Result Http.Error (List Feedback) -> msg) -> Cmd msg
getFeedbacks token collaboratorId toMsg =
    let
        url =
            case collaboratorId of
                Just id ->
                    "/feedbacks?collaborator_id=" ++ id

                Nothing ->
                    "/feedbacks"
    in
    authGet token url (Decode.list feedbackDecoder) toMsg


{-| The create/update payload for a feedback. Dates are held as `YYYY-MM-DD`
strings (from `<input type="date">`) and converted to RFC3339 on encode; the
optional fields are plain strings where empty means "omit".
-}
type alias FeedbackForm =
    { collaboratorId : String
    , feedbackDate : String
    , nextFeedbackDate : String
    , status : String
    , observation : String
    , observationPrivate : String
    }


{-| A blank feedback form bound to a collaborator.
-}
emptyFeedbackForm : String -> FeedbackForm
emptyFeedbackForm collaboratorId =
    { collaboratorId = collaboratorId
    , feedbackDate = ""
    , nextFeedbackDate = ""
    , status = ""
    , observation = ""
    , observationPrivate = ""
    }


{-| Pre-fills a feedback form from an existing feedback (for editing). The date
fields keep only the `YYYY-MM-DD` part.
-}
feedbackFormFromFeedback : Feedback -> FeedbackForm
feedbackFormFromFeedback feedback =
    { collaboratorId = feedback.collaboratorId
    , feedbackDate = String.left 10 feedback.feedbackDate
    , nextFeedbackDate = Maybe.withDefault "" (Maybe.map (String.left 10) feedback.nextFeedbackDate)
    , status = Maybe.withDefault "" feedback.status
    , observation = Maybe.withDefault "" feedback.observation
    , observationPrivate = Maybe.withDefault "" feedback.observationPrivate
    }


{-| Encodes a feedback form: `collaborator_id` and the RFC3339 `feedback_date` are
always present; the optional next date (also RFC3339) and the text fields are
included only when non-blank.
-}
encodeFeedbackForm : FeedbackForm -> Encode.Value
encodeFeedbackForm form =
    let
        nextDate =
            if String.trim form.nextFeedbackDate == "" then
                ""

            else
                toRfc3339 form.nextFeedbackDate
    in
    Encode.object
        (( "collaborator_id", Encode.string form.collaboratorId )
            :: ( "feedback_date", Encode.string (toRfc3339 form.feedbackDate) )
            :: List.filterMap optionalPair
                [ ( "next_feedback_date", nextDate )
                , ( "status", form.status )
                , ( "expectation_contract_observation", form.observation )
                , ( "expectation_contract_observation_private", form.observationPrivate )
                ]
        )


{-| Turns a `YYYY-MM-DD` date into the start-of-day UTC RFC3339 the backend parses.
-}
toRfc3339 : String -> String
toRfc3339 date =
    date ++ "T00:00:00Z"


{-| `POST /feedbacks` — creates a feedback.
-}
createFeedback : String -> FeedbackForm -> (Result Http.Error Feedback -> msg) -> Cmd msg
createFeedback token form toMsg =
    authRequest token
        "POST"
        "/feedbacks"
        (Http.jsonBody (encodeFeedbackForm form))
        (Http.expectJson toMsg feedbackDecoder)


{-| `PATCH /feedbacks/{id}` — updates a feedback.
-}
updateFeedback : String -> String -> FeedbackForm -> (Result Http.Error Feedback -> msg) -> Cmd msg
updateFeedback token id form toMsg =
    authRequest token
        "PATCH"
        ("/feedbacks/" ++ id)
        (Http.jsonBody (encodeFeedbackForm form))
        (Http.expectJson toMsg feedbackDecoder)


{-| `DELETE /feedbacks/{id}` — deactivates a feedback (soft delete; backend replies
`204`).
-}
deleteFeedback : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteFeedback token id toMsg =
    authRequest token "DELETE" ("/feedbacks/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)



-- EXPECTATION CONTRACT ITEMS


{-| One item of a feedback's expectation contract: a `goal` or `behavior`
checklist line (`kind`) with an optional description and a `done` flag.
-}
type alias ExpectationItem =
    { id : String
    , feedbackId : String
    , kind : String
    , description : Maybe String
    , done : Bool
    , active : Bool
    }


{-| Decodes a single expectation-contract item.
-}
expectationItemDecoder : Decoder ExpectationItem
expectationItemDecoder =
    Decode.succeed ExpectationItem
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "feedback_id" Decode.string)
        |> andMap (Decode.field "kind" Decode.string)
        |> andMap (optionalString "description")
        |> andMap (Decode.field "done" Decode.bool)
        |> andMap (Decode.field "active" Decode.bool)


{-| `GET /expectation-items?feedback_id=` — the items of one feedback (both kinds).
-}
getExpectationItems : String -> String -> (Result Http.Error (List ExpectationItem) -> msg) -> Cmd msg
getExpectationItems token feedbackId toMsg =
    authGet token
        ("/expectation-items?feedback_id=" ++ feedbackId)
        (Decode.list expectationItemDecoder)
        toMsg


{-| The create/update payload for an expectation-contract item.
-}
type alias ExpectationItemForm =
    { feedbackId : String
    , kind : String
    , description : String
    , done : Bool
    }


{-| Encodes an item form: `feedback_id`, `kind` and `done` are always present; the
description is included only when non-blank.
-}
encodeExpectationItemForm : ExpectationItemForm -> Encode.Value
encodeExpectationItemForm form =
    Encode.object
        (( "feedback_id", Encode.string form.feedbackId )
            :: ( "kind", Encode.string form.kind )
            :: ( "done", Encode.bool form.done )
            :: List.filterMap optionalPair [ ( "description", form.description ) ]
        )


{-| `POST /expectation-items` — creates an item under a feedback.
-}
createExpectationItem : String -> ExpectationItemForm -> (Result Http.Error ExpectationItem -> msg) -> Cmd msg
createExpectationItem token form toMsg =
    authRequest token
        "POST"
        "/expectation-items"
        (Http.jsonBody (encodeExpectationItemForm form))
        (Http.expectJson toMsg expectationItemDecoder)


{-| `PATCH /expectation-items/{id}` — updates an item (e.g. toggling `done`).
-}
updateExpectationItem : String -> String -> ExpectationItemForm -> (Result Http.Error ExpectationItem -> msg) -> Cmd msg
updateExpectationItem token id form toMsg =
    authRequest token
        "PATCH"
        ("/expectation-items/" ++ id)
        (Http.jsonBody (encodeExpectationItemForm form))
        (Http.expectJson toMsg expectationItemDecoder)


{-| `DELETE /expectation-items/{id}` — deactivates an item (soft delete; backend
replies `204`).
-}
deleteExpectationItem : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteExpectationItem token id toMsg =
    authRequest token "DELETE" ("/expectation-items/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)



-- FEEDBACK BEHAVIORS (scored DISC-values lines)


{-| A scored behavior line of a feedback: a value + behavior description, optional
observation/instruction, and an integer score.
-}
type alias FeedbackBehavior =
    { id : String
    , feedbackId : String
    , valueDescription : String
    , behaviorDescription : String
    , behaviorObs : Maybe String
    , valueInstruction : Maybe String
    , score : Int
    , active : Bool
    }


{-| Decodes a single feedback behavior.
-}
feedbackBehaviorDecoder : Decoder FeedbackBehavior
feedbackBehaviorDecoder =
    Decode.succeed FeedbackBehavior
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "feedback_id" Decode.string)
        |> andMap (Decode.field "value_description" Decode.string)
        |> andMap (Decode.field "behavior_description" Decode.string)
        |> andMap (optionalString "behavior_obs")
        |> andMap (optionalString "value_instruction")
        |> andMap (Decode.field "score" Decode.int)
        |> andMap (Decode.field "active" Decode.bool)


{-| `GET /feedback-behaviors?feedback_id=` — the scored behaviors of one feedback.
-}
getFeedbackBehaviors : String -> String -> (Result Http.Error (List FeedbackBehavior) -> msg) -> Cmd msg
getFeedbackBehaviors token feedbackId toMsg =
    authGet token
        ("/feedback-behaviors?feedback_id=" ++ feedbackId)
        (Decode.list feedbackBehaviorDecoder)
        toMsg


{-| The create/update payload for a scored behavior.
-}
type alias FeedbackBehaviorForm =
    { feedbackId : String
    , valueDescription : String
    , behaviorDescription : String
    , behaviorObs : String
    , valueInstruction : String
    , score : Int
    }


{-| A blank behavior form bound to a feedback (score starts at 0).
-}
emptyFeedbackBehaviorForm : String -> FeedbackBehaviorForm
emptyFeedbackBehaviorForm feedbackId =
    { feedbackId = feedbackId
    , valueDescription = ""
    , behaviorDescription = ""
    , behaviorObs = ""
    , valueInstruction = ""
    , score = 0
    }


{-| Pre-fills a behavior form from an existing behavior (for editing).
-}
feedbackBehaviorFormFromBehavior : FeedbackBehavior -> FeedbackBehaviorForm
feedbackBehaviorFormFromBehavior behavior =
    { feedbackId = behavior.feedbackId
    , valueDescription = behavior.valueDescription
    , behaviorDescription = behavior.behaviorDescription
    , behaviorObs = Maybe.withDefault "" behavior.behaviorObs
    , valueInstruction = Maybe.withDefault "" behavior.valueInstruction
    , score = behavior.score
    }


{-| Encodes a behavior form: `feedback_id`, the two descriptions and the integer
`score` are always present; the observation and instruction are included only when
non-blank.
-}
encodeFeedbackBehaviorForm : FeedbackBehaviorForm -> Encode.Value
encodeFeedbackBehaviorForm form =
    Encode.object
        (( "feedback_id", Encode.string form.feedbackId )
            :: ( "value_description", Encode.string form.valueDescription )
            :: ( "behavior_description", Encode.string form.behaviorDescription )
            :: ( "score", Encode.int form.score )
            :: List.filterMap optionalPair
                [ ( "behavior_obs", form.behaviorObs )
                , ( "value_instruction", form.valueInstruction )
                ]
        )


{-| `POST /feedback-behaviors` — creates a scored behavior under a feedback.
-}
createFeedbackBehavior : String -> FeedbackBehaviorForm -> (Result Http.Error FeedbackBehavior -> msg) -> Cmd msg
createFeedbackBehavior token form toMsg =
    authRequest token
        "POST"
        "/feedback-behaviors"
        (Http.jsonBody (encodeFeedbackBehaviorForm form))
        (Http.expectJson toMsg feedbackBehaviorDecoder)


{-| `PATCH /feedback-behaviors/{id}` — updates a scored behavior.
-}
updateFeedbackBehavior : String -> String -> FeedbackBehaviorForm -> (Result Http.Error FeedbackBehavior -> msg) -> Cmd msg
updateFeedbackBehavior token id form toMsg =
    authRequest token
        "PATCH"
        ("/feedback-behaviors/" ++ id)
        (Http.jsonBody (encodeFeedbackBehaviorForm form))
        (Http.expectJson toMsg feedbackBehaviorDecoder)


{-| `DELETE /feedback-behaviors/{id}` — deactivates a scored behavior (soft delete;
backend replies `204`).
-}
deleteFeedbackBehavior : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteFeedbackBehavior token id toMsg =
    authRequest token "DELETE" ("/feedback-behaviors/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)



-- ANNOTATIONS (quick scored notes)


{-| A quick scored note about a collaborator. The `note_date` is the raw RFC3339
string (the UI shows the date part). The second score, amount-of-days, notes and
observation are optional. `period_start_date` and `recorded_on_mobile` are not
surfaced in the UI.
-}
type alias Annotation =
    { id : String
    , collaboratorId : String
    , noteDate : String
    , score1Number : Int
    , score1Type : String
    , score1Description : Maybe String
    , askAmountDays : Bool
    , score2Number : Maybe Int
    , score2Type : Maybe String
    , score2Description : Maybe String
    , amountDays : Maybe Int
    , mainNote : Maybe String
    , observation : Maybe String
    , active : Bool
    }


{-| A nullable/absent integer field, decoded as `Maybe Int`.
-}
optionalIntField : String -> Decoder (Maybe Int)
optionalIntField name =
    Decode.maybe (Decode.field name Decode.int)


{-| Decodes a single annotation (ignoring fields the UI does not use).
-}
annotationDecoder : Decoder Annotation
annotationDecoder =
    Decode.succeed Annotation
        |> andMap (Decode.field "id" Decode.string)
        |> andMap (Decode.field "collaborator_id" Decode.string)
        |> andMap (Decode.field "note_date" Decode.string)
        |> andMap (Decode.field "score1_number" Decode.int)
        |> andMap (Decode.field "score1_type" Decode.string)
        |> andMap (optionalString "score1_description")
        |> andMap (Decode.field "ask_amount_days" Decode.bool)
        |> andMap (optionalIntField "score2_number")
        |> andMap (optionalString "score2_type")
        |> andMap (optionalString "score2_description")
        |> andMap (optionalIntField "amount_days")
        |> andMap (optionalString "main_note")
        |> andMap (optionalString "observation")
        |> andMap (Decode.field "active" Decode.bool)


{-| `GET /annotations` with the session token; an optional collaborator id narrows
the list to one collaborator.
-}
getAnnotations : String -> Maybe String -> (Result Http.Error (List Annotation) -> msg) -> Cmd msg
getAnnotations token collaboratorId toMsg =
    let
        url =
            case collaboratorId of
                Just id ->
                    "/annotations?collaborator_id=" ++ id

                Nothing ->
                    "/annotations"
    in
    authGet token url (Decode.list annotationDecoder) toMsg


{-| The create/update payload for an annotation. The optional integer fields
(`score2Number`, `amountDays`) are held as strings (empty/non-numeric means omit);
`score1Number` is always sent.
-}
type alias AnnotationForm =
    { collaboratorId : String
    , noteDate : String
    , score1Number : Int
    , score1Type : String
    , score1Description : String
    , askAmountDays : Bool
    , amountDays : String
    , score2Number : String
    , score2Type : String
    , score2Description : String
    , mainNote : String
    , observation : String
    }


{-| A blank annotation form bound to a collaborator.
-}
emptyAnnotationForm : String -> AnnotationForm
emptyAnnotationForm collaboratorId =
    { collaboratorId = collaboratorId
    , noteDate = ""
    , score1Number = 0
    , score1Type = ""
    , score1Description = ""
    , askAmountDays = False
    , amountDays = ""
    , score2Number = ""
    , score2Type = ""
    , score2Description = ""
    , mainNote = ""
    , observation = ""
    }


{-| Pre-fills an annotation form from an existing annotation (for editing).
-}
annotationFormFromAnnotation : Annotation -> AnnotationForm
annotationFormFromAnnotation annotation =
    { collaboratorId = annotation.collaboratorId
    , noteDate = String.left 10 annotation.noteDate
    , score1Number = annotation.score1Number
    , score1Type = annotation.score1Type
    , score1Description = Maybe.withDefault "" annotation.score1Description
    , askAmountDays = annotation.askAmountDays
    , amountDays = maybeIntToString annotation.amountDays
    , score2Number = maybeIntToString annotation.score2Number
    , score2Type = Maybe.withDefault "" annotation.score2Type
    , score2Description = Maybe.withDefault "" annotation.score2Description
    , mainNote = Maybe.withDefault "" annotation.mainNote
    , observation = Maybe.withDefault "" annotation.observation
    }


maybeIntToString : Maybe Int -> String
maybeIntToString value =
    Maybe.withDefault "" (Maybe.map String.fromInt value)


{-| A `(key, value)` integer field, dropped when the raw string is not a number.
-}
intPair : String -> String -> List ( String, Encode.Value )
intPair key raw =
    case String.toInt (String.trim raw) of
        Just number ->
            [ ( key, Encode.int number ) ]

        Nothing ->
            []


{-| Encodes an annotation form: `collaborator_id`, the RFC3339 `note_date`,
`score1_number`, `score1_type` and `ask_amount_days` are always present; the second
score, the conditional `amount_days` (only when `ask_amount_days`) and the text
fields are included only when set.
-}
encodeAnnotationForm : AnnotationForm -> Encode.Value
encodeAnnotationForm form =
    let
        amountDaysFields =
            if form.askAmountDays then
                intPair "amount_days" form.amountDays

            else
                []
    in
    Encode.object
        ([ ( "collaborator_id", Encode.string form.collaboratorId )
         , ( "note_date", Encode.string (toRfc3339 form.noteDate) )
         , ( "score1_number", Encode.int form.score1Number )
         , ( "score1_type", Encode.string form.score1Type )
         , ( "ask_amount_days", Encode.bool form.askAmountDays )
         ]
            ++ intPair "score2_number" form.score2Number
            ++ amountDaysFields
            ++ List.filterMap optionalPair
                [ ( "score1_description", form.score1Description )
                , ( "score2_type", form.score2Type )
                , ( "score2_description", form.score2Description )
                , ( "main_note", form.mainNote )
                , ( "observation", form.observation )
                ]
        )


{-| `POST /annotations` — creates an annotation.
-}
createAnnotation : String -> AnnotationForm -> (Result Http.Error Annotation -> msg) -> Cmd msg
createAnnotation token form toMsg =
    authRequest token
        "POST"
        "/annotations"
        (Http.jsonBody (encodeAnnotationForm form))
        (Http.expectJson toMsg annotationDecoder)


{-| `PATCH /annotations/{id}` — updates an annotation.
-}
updateAnnotation : String -> String -> AnnotationForm -> (Result Http.Error Annotation -> msg) -> Cmd msg
updateAnnotation token id form toMsg =
    authRequest token
        "PATCH"
        ("/annotations/" ++ id)
        (Http.jsonBody (encodeAnnotationForm form))
        (Http.expectJson toMsg annotationDecoder)


{-| `DELETE /annotations/{id}` — deactivates an annotation (soft delete; backend
replies `204`).
-}
deleteAnnotation : String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteAnnotation token id toMsg =
    authRequest token "DELETE" ("/annotations/" ++ id) Http.emptyBody (Http.expectWhatever toMsg)
