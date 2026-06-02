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
        , test "roleDecoder reads id, name and active, ignoring extra fields" <|
            \_ ->
                "{\"id\":\"r1\",\"name\":\"Backend Engineer\",\"objective\":\"Build\",\"active\":true}"
                    |> Decode.decodeString Api.roleDecoder
                    |> Expect.equal (Ok { id = "r1", name = "Backend Engineer", active = True })
        , test "collaboratorDecoder reads a present email" <|
            \_ ->
                "{\"id\":\"c1\",\"name\":\"Alice\",\"email\":\"alice@acme.test\",\"is_manager\":true}"
                    |> Decode.decodeString Api.collaboratorDecoder
                    |> Expect.equal
                        (Ok
                            { id = "c1"
                            , name = "Alice"
                            , email = Just "alice@acme.test"
                            , isManager = True
                            }
                        )
        , test "collaboratorDecoder reads a null email as Nothing" <|
            \_ ->
                "{\"id\":\"c2\",\"name\":\"Bob\",\"email\":null,\"is_manager\":false}"
                    |> Decode.decodeString Api.collaboratorDecoder
                    |> Expect.equal
                        (Ok
                            { id = "c2"
                            , name = "Bob"
                            , email = Nothing
                            , isManager = False
                            }
                        )
        , test "a list decoder reads an empty array" <|
            \_ ->
                "[]"
                    |> Decode.decodeString (Decode.list Api.sectorDecoder)
                    |> Expect.equal (Ok [])
        ]
