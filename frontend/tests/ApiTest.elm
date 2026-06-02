module ApiTest exposing (suite)

{-| Unit tests for the API boundary's pure parts: the login request encoder and
the login response decoder. No HTTP or server is involved.
-}

import Api
import Expect
import Json.Decode as Decode
import Json.Encode as Encode
import Test exposing (Test, describe, test)


suite : Test
suite =
    describe "Api"
        [ test "encodeCredentials produces the login JSON body" <|
            \_ ->
                Api.encodeCredentials { email = "admin@acme.test", password = "s3cret-pass" }
                    |> Encode.encode 0
                    |> Expect.equal "{\"email\":\"admin@acme.test\",\"password\":\"s3cret-pass\"}"
        , test "loginResponseDecoder reads token and token_type" <|
            \_ ->
                "{\"token\":\"abc.def.ghi\",\"token_type\":\"Bearer\"}"
                    |> Decode.decodeString Api.loginResponseDecoder
                    |> Expect.equal (Ok { token = "abc.def.ghi", tokenType = "Bearer" })
        , test "loginResponseDecoder fails when a field is missing" <|
            \_ ->
                "{\"token\":\"abc.def.ghi\"}"
                    |> Decode.decodeString Api.loginResponseDecoder
                    |> Result.toMaybe
                    |> Expect.equal Nothing
        , test "sectorDecoder reads id, name and active" <|
            \_ ->
                "{\"id\":\"s1\",\"name\":\"Engineering\",\"active\":true}"
                    |> Decode.decodeString Api.sectorDecoder
                    |> Expect.equal (Ok { id = "s1", name = "Engineering", active = True })
        , test "roleDecoder reads id, name, active and the optional description fields" <|
            \_ ->
                "{\"id\":\"r1\",\"name\":\"Backend Engineer\",\"profile_suggestion\":null,\"objective\":\"Build\",\"requirement_education\":null,\"requirement_experience\":null,\"requirement_attention\":null,\"requirement_knowledge\":null,\"requirement_skill\":null,\"requirement_attitude\":null,\"requirement_delivery\":null,\"observation\":null,\"active\":true}"
                    |> Decode.decodeString Api.roleDecoder
                    |> Expect.equal
                        (Ok
                            { id = "r1"
                            , name = "Backend Engineer"
                            , profileSuggestion = Nothing
                            , objective = Just "Build"
                            , requirementEducation = Nothing
                            , requirementExperience = Nothing
                            , requirementAttention = Nothing
                            , requirementKnowledge = Nothing
                            , requirementSkill = Nothing
                            , requirementAttitude = Nothing
                            , requirementDelivery = Nothing
                            , observation = Nothing
                            , active = True
                            }
                        )
        , test "collaboratorDecoder reads a present email and references" <|
            \_ ->
                "{\"id\":\"c1\",\"name\":\"Alice\",\"sector_id\":\"s1\",\"role_id\":\"r1\",\"manager_id\":null,\"whatsapp\":\"+55\",\"email\":\"alice@acme.test\",\"is_manager\":true}"
                    |> Decode.decodeString Api.collaboratorDecoder
                    |> Expect.equal
                        (Ok
                            { id = "c1"
                            , name = "Alice"
                            , sectorId = Just "s1"
                            , roleId = Just "r1"
                            , managerId = Nothing
                            , whatsapp = Just "+55"
                            , email = Just "alice@acme.test"
                            , isManager = True
                            }
                        )
        , test "collaboratorDecoder reads null/absent optional fields as Nothing" <|
            \_ ->
                "{\"id\":\"c2\",\"name\":\"Bob\",\"email\":null,\"is_manager\":false}"
                    |> Decode.decodeString Api.collaboratorDecoder
                    |> Expect.equal
                        (Ok
                            { id = "c2"
                            , name = "Bob"
                            , sectorId = Nothing
                            , roleId = Nothing
                            , managerId = Nothing
                            , whatsapp = Nothing
                            , email = Nothing
                            , isManager = False
                            }
                        )
        , test "a list decoder reads an empty array" <|
            \_ ->
                "[]"
                    |> Decode.decodeString (Decode.list Api.sectorDecoder)
                    |> Expect.equal (Ok [])
        , test "encodeSectorForm produces the sector JSON body" <|
            \_ ->
                Api.encodeSectorForm { name = "Engineering" }
                    |> Encode.encode 0
                    |> Expect.equal "{\"name\":\"Engineering\"}"
        , test "encodeRoleForm includes name and only the non-empty optional fields" <|
            \_ ->
                Api.encodeRoleForm
                    { name = "Engineer"
                    , profileSuggestion = ""
                    , objective = "Build"
                    , requirementEducation = ""
                    , requirementExperience = ""
                    , requirementAttention = ""
                    , requirementKnowledge = ""
                    , requirementSkill = "Rust"
                    , requirementAttitude = ""
                    , requirementDelivery = ""
                    , observation = ""
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"name\":\"Engineer\",\"objective\":\"Build\",\"requirement_skill\":\"Rust\"}"
        , test "encodeRoleForm of a name-only form omits every optional field" <|
            \_ ->
                Api.encodeRoleForm
                    { name = "Engineer"
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
                    |> Encode.encode 0
                    |> Expect.equal "{\"name\":\"Engineer\"}"
        , test "encodeCollaboratorForm always carries name and is_manager, plus chosen references" <|
            \_ ->
                Api.encodeCollaboratorForm
                    { name = "Alice"
                    , sectorId = "s1"
                    , roleId = ""
                    , managerId = ""
                    , whatsapp = ""
                    , email = "alice@acme.test"
                    , isManager = True
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"name\":\"Alice\",\"is_manager\":true,\"sector_id\":\"s1\",\"email\":\"alice@acme.test\"}"
        , test "encodeCollaboratorForm omits every unset optional field" <|
            \_ ->
                Api.encodeCollaboratorForm
                    { name = "Bob"
                    , sectorId = ""
                    , roleId = ""
                    , managerId = ""
                    , whatsapp = ""
                    , email = ""
                    , isManager = False
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"name\":\"Bob\",\"is_manager\":false}"
        , test "feedbackDecoder reads the feedback fields" <|
            \_ ->
                "{\"id\":\"f1\",\"collaborator_id\":\"c1\",\"feedback_date\":\"2026-06-02T00:00:00+00:00\",\"next_feedback_date\":null,\"expectation_contract_observation\":null,\"expectation_contract_observation_private\":null,\"status\":\"open\",\"active\":true}"
                    |> Decode.decodeString Api.feedbackDecoder
                    |> Expect.equal
                        (Ok
                            { id = "f1"
                            , collaboratorId = "c1"
                            , feedbackDate = "2026-06-02T00:00:00+00:00"
                            , nextFeedbackDate = Nothing
                            , observation = Nothing
                            , observationPrivate = Nothing
                            , status = Just "open"
                            , active = True
                            }
                        )
        , test "encodeFeedbackForm always sends collaborator_id and an RFC3339 date, plus the non-empty fields" <|
            \_ ->
                Api.encodeFeedbackForm
                    { collaboratorId = "c1"
                    , feedbackDate = "2026-06-02"
                    , nextFeedbackDate = ""
                    , status = "open"
                    , observation = ""
                    , observationPrivate = ""
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"collaborator_id\":\"c1\",\"feedback_date\":\"2026-06-02T00:00:00Z\",\"status\":\"open\"}"
        , test "encodeFeedbackForm converts the optional next date to RFC3339 and includes observations" <|
            \_ ->
                Api.encodeFeedbackForm
                    { collaboratorId = "c1"
                    , feedbackDate = "2026-06-02"
                    , nextFeedbackDate = "2026-09-01"
                    , status = ""
                    , observation = "Great progress"
                    , observationPrivate = "Private note"
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"collaborator_id\":\"c1\",\"feedback_date\":\"2026-06-02T00:00:00Z\",\"next_feedback_date\":\"2026-09-01T00:00:00Z\",\"expectation_contract_observation\":\"Great progress\",\"expectation_contract_observation_private\":\"Private note\"}"
        , test "expectationItemDecoder reads the item fields" <|
            \_ ->
                "{\"id\":\"i1\",\"feedback_id\":\"f1\",\"kind\":\"goal\",\"description\":\"Ship the SDK\",\"done\":false,\"active\":true}"
                    |> Decode.decodeString Api.expectationItemDecoder
                    |> Expect.equal
                        (Ok
                            { id = "i1"
                            , feedbackId = "f1"
                            , kind = "goal"
                            , description = Just "Ship the SDK"
                            , done = False
                            , active = True
                            }
                        )
        , test "encodeExpectationItemForm always sends feedback_id, kind and done, plus a non-empty description" <|
            \_ ->
                Api.encodeExpectationItemForm
                    { feedbackId = "f1", kind = "goal", description = "Ship the SDK", done = False }
                    |> Encode.encode 0
                    |> Expect.equal "{\"feedback_id\":\"f1\",\"kind\":\"goal\",\"done\":false,\"description\":\"Ship the SDK\"}"
        , test "encodeExpectationItemForm omits an empty description" <|
            \_ ->
                Api.encodeExpectationItemForm
                    { feedbackId = "f1", kind = "behavior", description = "", done = True }
                    |> Encode.encode 0
                    |> Expect.equal "{\"feedback_id\":\"f1\",\"kind\":\"behavior\",\"done\":true}"
        , test "feedbackBehaviorDecoder reads the scored-behavior fields" <|
            \_ ->
                "{\"id\":\"b1\",\"feedback_id\":\"f1\",\"value_description\":\"Leadership\",\"behavior_description\":\"Delegates clearly\",\"behavior_obs\":null,\"value_instruction\":null,\"score\":4,\"active\":true}"
                    |> Decode.decodeString Api.feedbackBehaviorDecoder
                    |> Expect.equal
                        (Ok
                            { id = "b1"
                            , feedbackId = "f1"
                            , valueDescription = "Leadership"
                            , behaviorDescription = "Delegates clearly"
                            , behaviorObs = Nothing
                            , valueInstruction = Nothing
                            , score = 4
                            , active = True
                            }
                        )
        , test "encodeFeedbackBehaviorForm always sends the required fields and the integer score" <|
            \_ ->
                Api.encodeFeedbackBehaviorForm
                    { feedbackId = "f1"
                    , valueDescription = "Leadership"
                    , behaviorDescription = "Delegates clearly"
                    , behaviorObs = ""
                    , valueInstruction = ""
                    , score = 4
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"feedback_id\":\"f1\",\"value_description\":\"Leadership\",\"behavior_description\":\"Delegates clearly\",\"score\":4}"
        , test "encodeFeedbackBehaviorForm includes the optional observation and instruction" <|
            \_ ->
                Api.encodeFeedbackBehaviorForm
                    { feedbackId = "f1"
                    , valueDescription = "Leadership"
                    , behaviorDescription = "Delegates clearly"
                    , behaviorObs = "Observed in standups"
                    , valueInstruction = "Coach on delegation"
                    , score = 3
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"feedback_id\":\"f1\",\"value_description\":\"Leadership\",\"behavior_description\":\"Delegates clearly\",\"score\":3,\"behavior_obs\":\"Observed in standups\",\"value_instruction\":\"Coach on delegation\"}"
        , test "annotationDecoder reads the annotation fields" <|
            \_ ->
                "{\"id\":\"a1\",\"collaborator_id\":\"c1\",\"note_date\":\"2026-06-02T00:00:00+00:00\",\"score1_number\":2,\"score1_description\":null,\"score1_type\":\"positive\",\"ask_amount_days\":false,\"score2_number\":null,\"score2_description\":null,\"score2_type\":null,\"amount_days\":null,\"main_note\":null,\"period_start_date\":null,\"observation\":null,\"recorded_on_mobile\":false,\"active\":true}"
                    |> Decode.decodeString Api.annotationDecoder
                    |> Expect.equal
                        (Ok
                            { id = "a1"
                            , collaboratorId = "c1"
                            , noteDate = "2026-06-02T00:00:00+00:00"
                            , score1Number = 2
                            , score1Type = "positive"
                            , score1Description = Nothing
                            , askAmountDays = False
                            , score2Number = Nothing
                            , score2Type = Nothing
                            , score2Description = Nothing
                            , amountDays = Nothing
                            , mainNote = Nothing
                            , observation = Nothing
                            , active = True
                            }
                        )
        , test "encodeAnnotationForm sends the required fields and omits unset optionals" <|
            \_ ->
                Api.encodeAnnotationForm
                    { collaboratorId = "c1"
                    , noteDate = "2026-06-02"
                    , score1Number = 2
                    , score1Type = "positive"
                    , score1Description = ""
                    , askAmountDays = False
                    , amountDays = "7"
                    , score2Number = ""
                    , score2Type = ""
                    , score2Description = ""
                    , mainNote = ""
                    , observation = ""
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"collaborator_id\":\"c1\",\"note_date\":\"2026-06-02T00:00:00Z\",\"score1_number\":2,\"score1_type\":\"positive\",\"ask_amount_days\":false}"
        , test "encodeAnnotationForm includes a second score, the conditional amount_days and notes" <|
            \_ ->
                Api.encodeAnnotationForm
                    { collaboratorId = "c1"
                    , noteDate = "2026-06-02"
                    , score1Number = 2
                    , score1Type = "positive"
                    , score1Description = ""
                    , askAmountDays = True
                    , amountDays = "7"
                    , score2Number = "1"
                    , score2Type = "attention"
                    , score2Description = ""
                    , mainNote = "Talk again"
                    , observation = "Keep an eye"
                    }
                    |> Encode.encode 0
                    |> Expect.equal "{\"collaborator_id\":\"c1\",\"note_date\":\"2026-06-02T00:00:00Z\",\"score1_number\":2,\"score1_type\":\"positive\",\"ask_amount_days\":true,\"score2_number\":1,\"amount_days\":7,\"score2_type\":\"attention\",\"main_note\":\"Talk again\",\"observation\":\"Keep an eye\"}"
        ]
