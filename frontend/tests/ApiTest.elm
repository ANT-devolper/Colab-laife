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
        ]
