#!/bin/bash
HOST="http://localhost:3000/api/v1"
echo "1. Register/Login (User: 13800000007)"
PHONE="13800000007"
SMS_CODE="666666"

# Send login code (auto register)
curl -X POST $HOST/auth/send_code -H "Content-Type: application/json" -d "{\"phone\": \"$PHONE\", \"purpose\": \"login\"}"
# Login
LOGIN_RES=$(curl -s -X POST $HOST/auth/login_sms -H "Content-Type: application/json" -d "{\"phone\": \"$PHONE\", \"code\": \"$SMS_CODE\", \"agreement_accepted\": true}")
TOKEN=$(echo $LOGIN_RES | grep -o '"token":"[^"]*"' | cut -d'"' -f4 | head -1) # head -1 to avoid duplicates if any
USER_ID=$(echo $LOGIN_RES | grep -o '"id":"[^"]*"' | cut -d'"' -f4 | head -1)

echo "Token: $TOKEN"
echo "UserId: $USER_ID"

if [ -z "$TOKEN" ]; then
  echo "Login failed"
  exit 1
fi

echo "2. Change Password (Waiting 61s for rate limit...)"
sleep 61
curl -X POST $HOST/auth/send_code -H "Content-Type: application/json" -d "{\"phone\": \"$PHONE\", \"purpose\": \"change_password\"}"
curl -X POST $HOST/auth/change_password -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"code\": \"$SMS_CODE\", \"new_password\": \"newpass123\"}"

echo "3. Change Phone (Waiting 61s for rate limit...)"
sleep 61
NEW_PHONE="13800000008"
curl -X POST $HOST/auth/send_code -H "Content-Type: application/json" -d "{\"phone\": \"$PHONE\", \"purpose\": \"change_phone_old\"}"
curl -X POST $HOST/auth/send_code -H "Content-Type: application/json" -d "{\"phone\": \"$NEW_PHONE\", \"purpose\": \"change_phone_new\"}"
# Verify old phone
curl -X POST $HOST/users/verify_old_phone -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"code\": \"$SMS_CODE\"}"
# Change phone
curl -X POST $HOST/users/change_phone -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"new_phone\": \"$NEW_PHONE\", \"new_code\": \"$SMS_CODE\"}"

echo "4. Update Profile"
# Note: User ID might need to be fetched again if change_phone changes ID (it shouldn't) or if we want to be sure.
curl -X PUT $HOST/users/update_user/$USER_ID -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"location\": \"Hangzhou\", \"birthday\": \"2000-01-01\", \"gender\": \"male\"}"

echo "5. Get User"
curl -X GET $HOST/users/get_user/$USER_ID -H "Authorization: Bearer $TOKEN"
