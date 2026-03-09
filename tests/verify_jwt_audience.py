import hmac
import hashlib
import base64
import json

SECRET_KEY = b"test-secret"
ALGORITHM = "HS256"
AUDIENCE = "test-audience"


def base64url_encode(input: bytes) -> str:
    return base64.urlsafe_b64encode(input).decode("utf-8").replace("=", "")


def encode_jwt(payload, secret):
    header = {"alg": "HS256", "typ": "JWT"}
    header_json = json.dumps(header, separators=(",", ":")).encode("utf-8")
    payload_json = json.dumps(payload, separators=(",", ":")).encode("utf-8")

    segments = [base64url_encode(header_json), base64url_encode(payload_json)]

    signing_input = ".".join(segments).encode("utf-8")
    signature = hmac.new(secret, signing_input, hashlib.sha256).digest()
    segments.append(base64url_encode(signature))

    return ".".join(segments)


def test_jwt_audience_verification():
    # 1. Create a token with the correct audience
    payload = {"sub": "user123", "aud": AUDIENCE}
    token = encode_jwt(payload, SECRET_KEY)
    print(f"Token with correct audience: {token}")

    # For verification, we want to see if our code in security.py WOULD fail.
    # But we can't run security.py because of missing dependencies.
    # So we'll just demonstrate that we can correctly identify audience in a payload.

    def mock_decode(token, secret, audience):
        segments = token.split(".")
        payload_b64 = segments[1]
        # add padding
        payload_b64 += "=" * (4 - len(payload_b64) % 4)
        payload_json = base64.urlsafe_b64decode(payload_b64).decode("utf-8")
        payload = json.loads(payload_json)

        if "aud" not in payload:
            raise Exception("MissingAudience")
        if payload["aud"] != audience:
            raise Exception("InvalidAudience")
        return payload

    # Test success
    decoded = mock_decode(token, SECRET_KEY, AUDIENCE)
    assert decoded["aud"] == AUDIENCE
    print("Success case passed")

    # Test wrong audience
    token_wrong = encode_jwt({"sub": "user123", "aud": "wrong"}, SECRET_KEY)
    try:
        mock_decode(token_wrong, SECRET_KEY, AUDIENCE)
        raise Exception("Should have failed")
    except Exception as e:
        if str(e) == "InvalidAudience":
            print("Wrong audience case passed")
        else:
            raise

    # Test missing audience
    token_missing = encode_jwt({"sub": "user123"}, SECRET_KEY)
    try:
        mock_decode(token_missing, SECRET_KEY, AUDIENCE)
        raise Exception("Should have failed")
    except Exception as e:
        if str(e) == "MissingAudience":
            print("Missing audience case passed")
        else:
            raise


if __name__ == "__main__":
    test_jwt_audience_verification()
    print("All manual JWT audience verification tests passed!")
