# Ezra API - Route Documentation

Complete API route reference for the Ezra music management system.

## Base URL

```
Production: http://your-domain.com
Development: http://localhost:8080
```

## Table of Contents

- [Device Tokens (FCM Push Notifications)](#device-tokens-fcm-push-notifications)
- [Authentication](#authentication)
- [OTP Verification](#otp-verification)
- [User Management](#user-management)
- [Music](#music)
- [Events](#events)
- [Bookings](#bookings)
- [Favorites](#favorites)
- [Bands](#bands)
- [Donations](#donations)
- [Supporters](#supporters)
- [Churches](#churches)
- [Settings](#settings)
- [Notifications](#notifications)
- [Health Check](#health-check)

---

## Authentication

### Register

Create a new user account with email/password. **Requires verified OTP** for email verification.

**Endpoint:** `POST /register`  
**Authentication:** None (Public)

**Prerequisites:**

1. Send OTP to email: `POST /otp/send` with `purpose: "email_verification"`
2. Verify OTP: `POST /otp/verify` with the code received
3. Then register with the verified OTP code

**Request Body:**

```json
{
  "username": "johndoe",
  "fullname": "John Doe",
  "email": "john@example.com",
  "password": "password123",
  "otp_code": "123456"
}
```

**Validation Rules:**

- `username`: required, min 3 chars, max 100 chars
- `fullname`: required, min 1 char, max 100 chars
- `email`: required, valid email format, max 100 chars
- `password`: required, min 6 chars
- `otp_code`: required, exactly 6 digits, must be verified via `/otp/verify`

**Success Response (201):**

```json
{
  "message": "user registered successfully",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input or validation error
- **400 Bad Request:** Invalid or expired OTP code
- **400 Bad Request:** OTP not verified (must call `/otp/verify` first)
- **400 Bad Request:** Username or email already exists

```json
{
  "error": "invalid OTP code"
}
```

```json
{
  "error": "OTP not verified. Please verify OTP first via /otp/verify"
}
```

```json
{
  "error": "username already exists"
}
```

---

### Login

Authenticate with username/email and password to get JWT token. **Optional 2FA** with OTP.

**Endpoint:** `POST /login`  
**Authentication:** None (Public)

**Request Body (Basic Login):**

```json
{
  "username": "johndoe",
  "password": "password123"
}
```

Or use email:

```json
{
  "username": "john@example.com",
  "password": "password123"
}
```

**Request Body (Login with 2FA):**
For extra security, optionally use OTP-based two-factor authentication:

1. Send OTP to user's email: `POST /otp/send` with `purpose: "login"` and user's email
2. Verify OTP: `POST /otp/verify` with the code received
3. Then login with the verified OTP code

```json
{
  "username": "johndoe",
  "password": "password123",
  "otp_code": "123456"
}
```

**Success Response (200):**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses:**

- **401 Unauthorized:** Invalid credentials
- **400 Bad Request:** Invalid request format
- **400 Bad Request:** Invalid 2FA OTP code (if `otp_code` provided)
- **400 Bad Request:** OTP not verified (must call `/otp/verify` first)
- **400 Bad Request:** OTP expired

```json
{
  "error": "invalid credentials"
}
```

```json
{
  "error": "invalid 2FA OTP code"
}
```

**Note:** The `otp_code` field is optional. If omitted, login proceeds with username/password only. If provided, it must be a valid, verified OTP with `purpose: "login"`.

---

### Google Login

Authenticate using Google OAuth ID token.

**Endpoint:** `POST /auth/google`  
**Authentication:** None (Public)

**Request Body:**

```json
{
  "id_token": "google_id_token_here"
}
```

**Success Response (200):**

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Error Responses:**

- **401 Unauthorized:** Invalid Google ID token
- **400 Bad Request:** Invalid token format or missing claims

```json
{
  "error": "invalid Google ID token"
}
```

**Note:** This endpoint validates the Google ID token, extracts user information (sub, email, name, picture), and either creates a new user or logs in an existing user linked to that Google account.

---

## OTP Verification

The system uses One-Time Password (OTP) verification for email verification, password reset, and optional two-factor authentication (2FA). OTPs are sent via Gmail SMTP and expire after 10 minutes (configurable).

### Send OTP

Generate and send an OTP code to the specified email address.

**Endpoint:** `POST /otp/send`  
**Authentication:** None (Public)

**Request Body:**

```json
{
  "email": "user@example.com",
  "purpose": "email_verification"
}
```

**Purpose Options:**

- `email_verification` - Verify email before user registration
- `password_reset` - Reset forgotten password
- `login` - Two-factor authentication (2FA) for login

**Validation Rules:**

- `email`: required, valid email format
- `purpose`: required, must be one of the allowed purposes

**Success Response (200):**

```json
{
  "message": "OTP sent successfully to your email",
  "email": "user@example.com",
  "expires_at": "2025-10-30T10:45:00Z"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid email format

```json
{
  "error": "invalid email format"
}
```

- **400 Bad Request:** Email already registered (for `email_verification` purpose only)

```json
{
  "error": "email already registered, please login or reset password"
}
```

- **400 Bad Request:** Email not found (for `login` and `password_reset` purposes)

```json
{
  "error": "email not found"
}
```

- **400 Bad Request:** Email not found (for `password_reset` purpose only)

```json
{
  "error": "email not found"
}
```

**Notes:**

- OTP code is 6 digits
- OTP expires after 10 minutes (default, configurable via `OTP_EXPIRY_MINUTES`)
- Any existing unverified OTPs for the same email and purpose are deleted before sending a new one
- Email is sent with a beautiful HTML template including the code and expiration warning

---

### Verify OTP

Verify the OTP code sent to the email address.

**Endpoint:** `POST /otp/verify`  
**Authentication:** None (Public)

**Request Body:**

```json
{
  "email": "user@example.com",
  "code": "123456",
  "purpose": "email_verification"
}
```

**Validation Rules:**

- `email`: required, valid email format
- `code`: required, exactly 6 digits
- `purpose`: required, must match the OTP purpose

**Success Response (200):**

```json
{
  "message": "OTP verified successfully",
  "data": {
    "email": "user@example.com",
    "purpose": "email_verification"
  }
}
```

**Error Responses:**

- **400 Bad Request:** Invalid or expired OTP

```json
{
  "error": "invalid or expired OTP"
}
```

- **400 Bad Request:** OTP already used

```json
{
  "error": "OTP already used"
}
```

- **400 Bad Request:** OTP has expired

```json
{
  "error": "OTP has expired"
}
```

**Notes:**

- After successful verification, the OTP is marked as `verified = true`
- OTP can only be verified once
- Expired OTPs (>10 minutes old) cannot be verified
- The verified OTP must be used within its validity period for the intended action

---

### Reset Password with OTP

Reset user password using a verified OTP code.

**Endpoint:** `POST /auth/reset-password`  
**Authentication:** None (Public)

**Request Body:**

```json
{
  "email": "user@example.com",
  "new_password": "newSecurePassword123",
  "otp_code": "654331"
}
```

**Validation Rules:**

- `email`: required, valid email format
- `new_password`: required, minimum 6 characters
- `otp_code`: required, exactly 6 digits

**Success Response (200):**

```json
{
  "message": "password reset successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid OTP code

```json
{
  "error": "invalid OTP code"
}
```

- **400 Bad Request:** OTP not verified

```json
{
  "error": "OTP not verified. Please verify OTP first"
}
```

- **400 Bad Request:** OTP has expired

```json
{
  "error": "OTP has expired. Please request a new one"
}
```

- **400 Bad Request:** User not found

```json
{
  "error": "user not found"
}
```

**Notes:**

- OTP must be verified (via `/otp/verify`) before using this endpoint
- OTP purpose must be `password_reset`
- Password is hashed using bcrypt before storage
- All existing user sessions are invalidated (user logged out from all devices)
- Used OTP is deleted after successful password reset

---

### OTP Workflow Examples

#### 1. Registration with Email Verification

```bash
# Step 1: Send OTP for email verification
curl -X POST http://localhost:8080/otp/send \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "purpose": "email_verification"
  }'

# Response: OTP sent to email

# Step 2: User receives email with 6-digit code (e.g., 123456)

# Step 3: Verify OTP
curl -X POST http://localhost:8080/otp/verify \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "code": "123456",
    "purpose": "email_verification"
  }'

# Response: OTP verified successfully

# Step 4: Register user with verified OTP code
curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "newuser",
    "fullname": "New User",
    "email": "newuser@example.com",
    "password": "securePassword123",
    "otp_code": "123456"
  }'

# Response: User registered with email_verified = true
```

#### 2. Password Reset Flow

```bash
# Step 1: Request password reset OTP
curl -X POST http://localhost:8080/otp/send \
  -H "Content-Type: application/json" \
  -d '{
    "email": "existing@example.com",
    "purpose": "password_reset"
  }'

# Step 2: Verify OTP from email
curl -X POST http://localhost:8080/otp/verify \
  -H "Content-Type: application/json" \
  -d '{
    "email": "existing@example.com",
    "code": "654331",
    "purpose": "password_reset"
  }'

# Step 3: Reset password with verified OTP
curl -X POST http://localhost:8080/auth/reset-password \
  -H "Content-Type: application/json" \
  -d '{
    "email": "existing@example.com",
    "new_password": "newPassword123",
    "otp_code": "654331"
  }'

# Response: Password reset successfully
# Note: All user sessions are invalidated
```

#### 3. Login with 2FA (Optional)

```bash
# Step 1: Request login OTP
curl -X POST http://localhost:8080/otp/send \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "purpose": "login"
  }'

# Step 2: Verify OTP from email
curl -X POST http://localhost:8080/otp/verify \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "code": "789012",
    "purpose": "login"
  }'

# Step 3: Login with password and verified OTP
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user@example.com",
    "password": "userPassword123",
    "otp_code": "789012"
  }'

# Response: JWT token with 2FA verification
```

**Note:** The `otp_code` in login is optional. If you want standard username/password login without 2FA, simply omit the `otp_code` field.

---

### OTP Configuration

**Environment Variables:**

```env
# Gmail SMTP Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-gmail-app-password
SMTP_FROM=your-email@gmail.com

# OTP Settings
OTP_EXPIRY_MINUTES=10
```

**Gmail App Password Setup:**

1. Go to https://myaccount.google.com/security
2. Enable 2-Step Verification
3. Go to https://myaccount.google.com/apppasswords
4. Generate app password for "Mail"
5. Copy the 16-character code (no spaces)
6. Use in `SMTP_PASSWORD` environment variable

---

### OTP Security Features

| Feature                  | Description                                 |
| ------------------------ | ------------------------------------------- |
| **Time-Limited**         | OTPs expire after 10 minutes (configurable) |
| **One-Time Use**         | Cannot reuse verified OTPs                  |
| **Purpose Isolation**    | Email verification OTP ≠ Password reset OTP |
| **Email Verification**   | Tracked in `users.email_verified` field     |
| **Session Invalidation** | All sessions logged out on password reset   |
| **Auto Cleanup**         | Old OTPs deleted when sending new ones      |
| **Bcrypt Hashing**       | Secure password storage                     |

---

### OTP Email Template

The OTP email includes:

- Professional header with branding
- Clear purpose message (email verification, password reset, or login)
- Large, easy-to-read 6-digit code
- Expiration warning (10 minutes)
- Security reminder if user didn't request it
- Responsive design for mobile devices

---

### OTP Database Schema

**OTPs Table:**

```sql
CREATE TABLE otps (
    id SERIAL PRIMARY KEY,
    email VARCHAR(100) NOT NULL,
    code VARCHAR(10) NOT NULL,
    purpose VARCHAR(50) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT now()
);

-- Indexes for performance
CREATE INDEX idx_otps_email ON otps(email);
CREATE INDEX idx_otps_purpose ON otps(purpose);
CREATE INDEX idx_otps_expires_at ON otps(expires_at);
CREATE INDEX idx_otps_email_purpose ON otps(email, purpose);
```

**Users Table (Email Verification):**

```sql
ALTER TABLE users ADD COLUMN email_verified BOOLEAN DEFAULT false;
```

---

### OTP Best Practices

**Security:**

- ✅ Always validate email format
- ✅ Check OTP expiration before use
- ✅ Verify OTP purpose matches intended action
- ✅ Invalidate sessions on password reset
- ✅ Delete used OTPs immediately
- ✅ Use HTTPS in production

**User Experience:**

- ✅ Show countdown timer (10 minutes)
- ✅ Provide "Resend OTP" option
- ✅ Clear error messages
- ✅ Success confirmation feedback
- ✅ Mobile-friendly email design

**Production:**

- ✅ Implement rate limiting (max 3 OTPs per hour per email)
- ✅ Monitor failed verification attempts
- ✅ Log OTP operations for security audits
- ✅ Set up email delivery monitoring
- ✅ Consider SMS alternative for critical operations

---

## User Management

### Logout

Invalidate the current user's JWT token.

**Endpoint:** `POST /api/logout`  
**Authentication:** Required (Bearer Token)

**Headers:**

```
Authorization: Bearer <your_jwt_token>
```

**Success Response (200):**

```json
{
  "message": "logged out successfully"
}
```

**Error Responses:**

- **401 Unauthorized:** User not authenticated
- **400 Bad Request:** Logout failed

---

### Delete User

Delete the current user's account and all associated data.

**Endpoint:** `DELETE /api/user`  
**Authentication:** Required (Bearer Token)

**Headers:**

```
Authorization: Bearer <your_jwt_token>
```

**Success Response (200):**

```json
{
  "message": "user deleted successfully"
}
```

**Error Responses:**

- **401 Unauthorized:** User not authenticated
- **400 Bad Request:** Deletion failed

**Note:** This is a permanent action. All user data including music, events, bookings, favorites, bands, and settings will be deleted due to CASCADE constraints.

---

## Music

### Create Music

Add a new music track to the system.

**Endpoint:** `POST /api/musics`  
**Authentication:** Required (Bearer Token)

**Headers:**

```
Authorization: Bearer <your_jwt_token>
Content-Type: application/json
```

**Request Body:**

```json
{
  "title": "Amazing Grace",
  "artist": "John Newton",
  "album": "Hymns of Grace",
  "genre": "Hymn",
  "duration": 240,
  "bpm": 80,
  "key": "G",
  "cover": "https://example.com/covers/amazing-grace.jpg",
  "lyrics": "Amazing grace, how sweet the sound...",
  "description": "A classic hymn about redemption"
}
```

**Validation Rules:**

- `title`: required, min 1 char, max 255 chars
- `artist`: optional, max 255 chars
- `album`: optional, max 255 chars
- `genre`: optional, max 100 chars
- `duration`: optional, integer (in seconds)
- `bpm`: optional, integer (beats per minute)
- `key`: optional, max 10 chars (musical key like "C", "Am", "G", etc.)
- `cover`: optional, max 255 chars
- `lyrics`: optional, text
- `description`: optional, text

**Success Response (201):**

```json
{
  "message": "music created successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input or validation error
- **401 Unauthorized:** User not authenticated

---

### Get All Music

Retrieve all music tracks with optional pagination.

**Endpoint:** `GET /api/musics`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1) - Page number
- `page_size` (optional, default: 20, max: 100) - Items per page

**Examples:**

- Get all: `GET /api/musics`
- With pagination: `GET /api/musics?page=1&page_size=20`

**Success Response (200) - Paginated:**

```json
{
  "data": [
    {
      "id": 1,
      "title": "Amazing Grace",
      "artist": "John Newton",
      "album": "Hymns of Grace",
      "genre": "Hymn",
      "duration": 240,
      "bpm": 80,
      "key": "G",
      "cover": "https://example.com/covers/amazing-grace.jpg",
      "lyrics": "Amazing grace, how sweet the sound...",
      "description": "A classic hymn about redemption",
      "user_id": 5,
      "created_at": "2025-10-25T10:30:00Z",
      "updated_at": "2025-10-25T10:30:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 5,
    "total_records": 95,
    "has_next_page": true,
    "has_prev_page": false
  }
}
```

**Success Response (200) - All Results:**

```json
[
  {
    "id": 1,
    "title": "Amazing Grace",
    "artist": "John Newton",
    "album": "Hymns of Grace",
    "genre": "Hymn",
    "duration": 240,
    "bpm": 80,
    "key": "G",
    "cover": "https://example.com/covers/amazing-grace.jpg",
    "lyrics": "Amazing grace, how sweet the sound...",
    "description": "A classic hymn about redemption",
    "user_id": 5,
    "created_at": "2025-10-25T10:30:00Z",
    "updated_at": "2025-10-25T10:30:00Z"
  }
]
```

---

### Get User's Music

Retrieve all music tracks created by the authenticated user.

**Endpoint:** `GET /api/musics/user`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
[
  {
    "id": 1,
    "title": "Amazing Grace",
    "artist": "John Newton",
    "album": "Hymns of Grace",
    "genre": "Hymn",
    "duration": 240,
    "bpm": 80,
    "key": "G",
    "cover": "https://example.com/covers/amazing-grace.jpg",
    "lyrics": "Amazing grace, how sweet the sound...",
    "description": "A classic hymn about redemption",
    "user_id": 5,
    "created_at": "2025-10-25T10:30:00Z",
    "updated_at": "2025-10-25T10:30:00Z"
  }
]
```

**Error Responses:**

- **401 Unauthorized:** User not authenticated
- **500 Internal Server Error:** Server error

---

### Get Music by ID

Retrieve a specific music track by its ID.

**Endpoint:** `GET /api/musics/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "id": 1,
  "title": "Amazing Grace",
  "artist": "John Newton",
  "album": "Hymns of Grace",
  "genre": "Hymn",
  "duration": 240,
  "bpm": 80,
  "key": "G",
  "cover": "https://example.com/covers/amazing-grace.jpg",
  "lyrics": "Amazing grace, how sweet the sound...",
  "description": "A classic hymn about redemption",
  "user_id": 5,
  "created_at": "2025-10-25T10:30:00Z",
  "updated_at": "2025-10-25T10:30:00Z"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid ID format
- **404 Not Found:** Music not found

```json
{
  "error": "music not found"
}
```

---

### Update Music

Update an existing music track. Only the owner can update.

**Endpoint:** `PUT /api/musics/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Request Body:**

```json
{
  "title": "Amazing Grace (Updated)",
  "artist": "John Newton",
  "album": "Hymns of Grace - Remastered",
  "genre": "Hymn",
  "duration": 245,
  "bpm": 82,
  "key": "G",
  "cover": "https://example.com/covers/amazing-grace-v2.jpg",
  "lyrics": "Amazing grace, how sweet the sound...",
  "description": "A classic hymn about redemption - updated version"
}
```

**Success Response (200):**

```json
{
  "message": "music updated successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input or user doesn't own this music
- **401 Unauthorized:** User not authenticated
- **404 Not Found:** Music not found

---

### Delete Music

Delete a music track. Only the owner can delete.

**Endpoint:** `DELETE /api/musics/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "message": "music deleted successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid ID or user doesn't own this music
- **401 Unauthorized:** User not authenticated
- **404 Not Found:** Music not found

---

## Events

### Create Event

Create a new event with optional music tracks.

**Endpoint:** `POST /api/events`  
**Authentication:** Required (Bearer Token)

**Note:** Creating an event automatically sends a broadcast notification to all users informing them about the new event. The event creator will not receive the notification themselves.

**Request Body:**

```json
{
  "title": "Sunday Worship Service",
  "content": "Weekly Sunday morning worship service",
  "cover": "https://example.com/events/sunday-worship.jpg",
  "location": "Main Church Auditorium",
  "start_time": "2025-11-01T09:00:00Z",
  "end_time": "2025-11-01T11:00:00Z",
  "music_ids": [1, 2, 3, 4]
}
```

**Validation Rules:**

- `title`: required, min 1 char, max 255 chars
- `content`: optional, text
- `cover`: optional, max 255 chars
- `location`: required
- `start_time`: required, ISO 8601 format
- `end_time`: required, ISO 8601 format
- `music_ids`: optional, array of music IDs

**Success Response (201):**

```json
{
  "id": 1,
  "title": "Sunday Worship Service",
  "content": "Weekly Sunday morning worship service",
  "cover": "https://example.com/events/sunday-worship.jpg",
  "location": "Main Church Auditorium",
  "start_time": "2025-11-01T09:00:00Z",
  "end_time": "2025-11-01T11:00:00Z",
  "user_id": 5,
  "musics": [
    {
      "id": 1,
      "title": "Amazing Grace",
      "artist": "John Newton",
      "genre": "Hymn",
      "cover": "..."
    }
  ],
  "created_at": "2025-10-29T10:30:00Z",
  "updated_at": "2025-10-29T10:30:00Z"
}
```

---

### Get All Events

Retrieve all events with optional pagination.

**Endpoint:** `GET /api/events`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20, max: 100)

**Success Response (200):** Similar to Get All Music, returns paginated event list with associated music tracks.

---

### Get User's Events

Retrieve events created by the authenticated user.

**Endpoint:** `GET /api/events/user`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

**Success Response (200):** Returns list of user's events with music.

---

### Get Event by ID

Retrieve a specific event with its associated music tracks.

**Endpoint:** `GET /api/events/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Event ID (integer)

**Success Response (200):** Returns event details including all associated music tracks.

---

### Update Event

Update an existing event. Only the owner can update.

**Endpoint:** `PUT /api/events/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Event ID (integer)

**Request Body:** Same format as Create Event

**Success Response (200):** Returns updated event with music.

---

### Delete Event

Delete an event. Only the owner can delete.

**Endpoint:** `DELETE /api/events/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Event ID (integer)

**Success Response (200):**

```json
{
  "message": "event deleted successfully"
}
```

---

## Bookings

### Create Booking

Register/book attendance for an event.

**Endpoint:** `POST /api/bookings`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "event_id": 1,
  "notes": "Looking forward to attending!"
}
```

**Success Response (201):**

```json
{
  "id": 1,
  "event_id": 1,
  "user_id": 5,
  "status": "pending",
  "notes": "Looking forward to attending!",
  "created_at": "2025-10-29T10:30:00Z",
  "updated_at": "2025-10-29T10:30:00Z"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input or duplicate booking
- **401 Unauthorized:** User not authenticated

---

### Get All Bookings

Retrieve all bookings in the system (typically admin use).

**Endpoint:** `GET /api/bookings`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20)

---

### Get User's Bookings

Retrieve all bookings made by the authenticated user.

**Endpoint:** `GET /api/bookings/user`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

**Success Response (200):** Returns paginated list of user's bookings.

---

### Get Bookings by Event

Retrieve all bookings for a specific event.

**Endpoint:** `GET /api/bookings/event/:event_id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `event_id` (required) - Event ID (integer)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

---

### Get Booking by ID

Retrieve a specific booking.

**Endpoint:** `GET /api/bookings/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Booking ID (integer)

---

### Update Booking

Update booking status or notes.

**Endpoint:** `PUT /api/bookings/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Booking ID (integer)

**Request Body:**

```json
{
  "status": "confirmed",
  "notes": "Attendance confirmed"
}
```

**Valid Status Values:**

- `pending` - Awaiting confirmation
- `confirmed` - Attendance confirmed
- `cancelled` - Booking cancelled

**Success Response (200):** Returns updated booking

---

### Delete Booking

Cancel/delete a booking. Only the booking owner can delete.

**Endpoint:** `DELETE /api/bookings/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Booking ID (integer)

**Success Response (200):**

```json
{
  "message": "booking deleted successfully"
}
```

---

## Favorites

### Get User's Favorites

Retrieve all music tracks favorited by the authenticated user.

**Endpoint:** `GET /api/favorites`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20)

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 1,
      "title": "Amazing Grace",
      "artist": "John Newton",
      "album": "Hymns of Grace",
      "genre": "Hymn",
      "duration": 240,
      "bpm": 80,
      "key": "G",
      "cover": "https://example.com/covers/amazing-grace.jpg",
      "lyrics": "Amazing grace, how sweet the sound...",
      "description": "A classic hymn about redemption",
      "user_id": 3,
      "created_at": "2025-10-25T10:30:00Z",
      "updated_at": "2025-10-25T10:30:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 2,
    "total_records": 35,
    "has_next_page": true,
    "has_prev_page": false
  }
}
```

---

### Add to Favorites

Add a music track to the user's favorites list.

**Endpoint:** `POST /api/favorites/music/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "message": "music added to favorites"
}
```

**Error Responses:**

- **400 Bad Request:** Music already in favorites or music doesn't exist
- **401 Unauthorized:** User not authenticated

---

### Remove from Favorites

Remove a music track from the user's favorites list.

**Endpoint:** `DELETE /api/favorites/music/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "message": "music removed from favorites"
}
```

**Error Responses:**

- **400 Bad Request:** Music not in favorites
- **401 Unauthorized:** User not authenticated

---

### Check if Favorite

Check whether a specific music track is in the user's favorites.

**Endpoint:** `GET /api/favorites/music/:id/check`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "is_favorite": true
}
```

Or:

```json
{
  "is_favorite": false
}
```

---

### Get Favorite Count

Get the total number of users who have favorited a specific music track.

**Endpoint:** `GET /api/favorites/music/:id/count`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "count": 42
}
```

**Note:** This is useful for displaying popularity metrics.

---

## Bands

### Create Band

Create a new band/music collection/library.

**Endpoint:** `POST /api/bands`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "name": "Worship Team A",
  "description": "Main worship team for Sunday services",
  "cover": "https://example.com/bands/worship-team-a.jpg",
  "is_public": true
}
```

**Validation Rules:**

- `name`: required, min 1 char, max 255 chars
- `description`: optional, text
- `cover`: optional, max 255 chars
- `is_public`: optional, boolean (default: false)

**Success Response (201):**

```json
{
  "id": 1,
  "name": "Worship Team A",
  "description": "Main worship team for Sunday services",
  "cover": "https://example.com/bands/worship-team-a.jpg",
  "is_public": true,
  "user_id": 5,
  "created_at": "2025-10-29T10:30:00Z",
  "updated_at": "2025-10-29T10:30:00Z"
}
```

---

### Get All Bands

Retrieve all bands with pagination.

**Endpoint:** `GET /api/bands`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20)

**Success Response (200):** Returns paginated list of all bands.

---

### Get User's Bands

Retrieve bands created by the authenticated user.

**Endpoint:** `GET /api/bands/user`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

---

### Get Public Bands

Retrieve all public bands (bands with `is_public = true`).

**Endpoint:** `GET /api/bands/public`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

---

### Get Band by ID

Retrieve a specific band's details.

**Endpoint:** `GET /api/bands/:id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Success Response (200):** Returns band details.

---

### Update Band

Update band information. Only the owner can update.

**Endpoint:** `PUT /api/bands/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Request Body:** Same format as Create Band

**Success Response (200):** Returns updated band.

---

### Delete Band

Delete a band. Only the owner can delete.

**Endpoint:** `DELETE /api/bands/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Success Response (200):**

```json
{
  "message": "band deleted successfully"
}
```

---

### Get Band Music

Retrieve all music tracks in a specific band.

**Endpoint:** `GET /api/bands/:id/musics`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 1,
      "title": "Amazing Grace",
      "artist": "John Newton",
      "album": "Hymns of Grace",
      "genre": "Hymn",
      "duration": 240,
      "bpm": 80,
      "key": "G",
      "cover": "https://example.com/covers/amazing-grace.jpg",
      "lyrics": "Amazing grace, how sweet the sound...",
      "description": "A classic hymn about redemption",
      "user_id": 5,
      "created_at": "2025-10-25T10:30:00Z",
      "updated_at": "2025-10-25T10:30:00Z"
    }
  ]
}
```

---

### Add Music to Band

Add one or more music tracks to a band. Only the owner can add.

**Endpoint:** `POST /api/bands/:id/musics`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Request Body:**

```json
{
  "music_ids": [1, 2, 3, 4, 5]
}
```

**Success Response (200):**

```json
{
  "message": "music added to band successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input, music doesn't exist, or already in band
- **401 Unauthorized:** User not authenticated or not band owner

---

### Remove Music from Band

Remove a music track from a band. Only the owner can remove.

**Endpoint:** `DELETE /api/bands/:id/musics/:music_id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Band ID (integer)
- `music_id` (required) - Music ID (integer)

**Success Response (200):**

```json
{
  "message": "music removed from band"
}
```

---

### Reorder Band Music

Change the display order of music tracks in a band. Only the owner can reorder.

**Endpoint:** `PUT /api/bands/:id/musics/reorder`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Request Body:**

```json
{
  "music_orders": [
    { "music_id": 3, "display_order": 1 },
    { "music_id": 1, "display_order": 2 },
    { "music_id": 5, "display_order": 3 },
    { "music_id": 2, "display_order": 4 }
  ]
}
```

**Success Response (200):**

```json
{
  "message": "music order updated successfully"
}
```

**Note:** This allows you to customize the order in which music appears in the band's playlist.

---

### Get Band Members

Retrieve all users who are members of a specific band.

**Endpoint:** `GET /api/bands/:id/members`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Band ID (integer)

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 5,
      "username": "johndoe",
      "fullname": "John Doe",
      "email": "john@example.com",
      "profile": "https://example.com/profiles/johndoe.jpg",
      "role": "user",
      "created_at": "2025-10-01T10:00:00Z"
    }
  ]
}
```

**Note:** Members are users who have their `band_id` set to this band.

---

## Donations

Support donations and sponsorships with integrated Payway by ABA payment processing.

### Features

- **Two donation types**: `donate` (QR payment) and `sponsor` (Card payment)
- **Two donor types**: `user` (authenticated) and `company` (anonymous)
- **One-call payment**: Create donation and get payment info immediately
- **QR code expiration**: 3-minute expiration for security
- **Event linking**: Donations can support app or specific events
- **Payment tracking**: Full transaction history and status
- **Statistics**: Comprehensive donation analytics

---

### Create Donation (One-Step Payment)

Create a donation and optionally get payment info immediately.

**Endpoint:** `POST /donations`  
**Authentication:** Optional (Required for user donations, public for company donations)

**Request Body (User Donation with QR Payment):**

```json
{
  "type": "donate",
  "donor_type": "user",
  "amount": 50.0,
  "currency": "USD",
  "message": "Happy to support!",
  "event_id": 5,
  "initiate_payment": true
}
```

**Request Body (Company Sponsorship with Card Payment):**

```json
{
  "type": "sponsor",
  "donor_type": "company",
  "company_name": "Tech Solutions Ltd",
  "company_email": "sponsor@techsolutions.com",
  "company_phone": "+855123456789",
  "amount": 5000.0,
  "currency": "USD",
  "message": "Proud to sponsor this event!",
  "event_id": 5,
  "initiate_payment": true
}
```

**Request Body (Using Supporter Profile):**

```json
{
  "type": "sponsor",
  "donor_type": "company",
  "supporter_id": 1,
  "amount": 5000.0,
  "currency": "USD",
  "message": "Proud to sponsor this event!",
  "event_id": 5,
  "initiate_payment": true
}
```

**Validation Rules:**

- `type`: required, `donate` or `sponsor`
- `donor_type`: required, `user`, `company`, `organization`, or `church`
- `amount`: required, must be > 0
- `currency`: required, `USD` or `KHR`
- `message`: optional, text
- `event_id`: optional, links donation to specific event (null = app support)
- `initiate_payment`: optional, boolean (if true, returns payment info immediately)
- `supporter_id`: optional, use existing supporter profile instead of inline info
- `company_name`: required if no `supporter_id` and `donor_type != user`
- `company_email`: required if no `supporter_id` and `donor_type != user`
- `company_phone`: optional

**Success Response (201) - QR Payment:**

```json
{
  "message": "donation created successfully. Please scan the QR code to complete payment",
  "data": {
    "id": 123,
    "type": "donate",
    "donor_type": "user",
    "amount": 50.0,
    "currency": "USD",
    "message": "Happy to support!",
    "status": "pending",
    "event_id": 5,
    "created_at": "2024-01-15T10:30:00Z",
    "payment_info": {
      "donation_id": 123,
      "transaction_id": "DON-123-1705318200",
      "qr_code": "data:image/png;base64,iVBORw0KGgoAAAA...",
      "payment_method": "qr",
      "expires_at": "2024-01-15T10:33:00Z",
      "expires_in_seconds": 180,
      "amount": "5000",
      "currency": "USD"
    }
  }
}
```

**Success Response (201) - Card Payment:**

```json
{
  "message": "donation created successfully. Please complete payment via the provided URL",
  "data": {
    "id": 124,
    "type": "sponsor",
    "donor_type": "company",
    "company_name": "Tech Solutions Ltd",
    "amount": 5000.0,
    "currency": "USD",
    "status": "pending",
    "event_id": 5,
    "created_at": "2024-01-15T10:30:00Z",
    "payment_info": {
      "donation_id": 124,
      "transaction_id": "DON-124-1705318300",
      "payment_url": "https://checkout.payway.com.kh/payment/abc123...",
      "payment_method": "card",
      "amount": "500000",
      "currency": "USD"
    }
  }
}
```

**Payment Methods by Type:**

- `donate` → QR Code payment (KHQR via ABA/Wing/Bakong)
- `sponsor` → Visa/Mastercard payment (redirects to Payway)

**Error Responses:**

- **400 Bad Request:** Invalid input or validation error
- **401 Unauthorized:** User authentication required for user donations

---

### Initiate Payment (Separate Step)

Generate payment info for an existing donation.

**Endpoint:** `POST /donations/:id/pay`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Success Response (200):** Same as payment_info above

**Error Responses:**

- **400 Bad Request:** Invalid ID or payment already completed
- **404 Not Found:** Donation not found

---

### Get All Donations

Retrieve donations with filtering and pagination.

**Endpoint:** `GET /donations`  
**Authentication:** None (Public)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20, max: 100)
- `type` (optional) - Filter by: `donate` or `sponsor`
- `donor_type` (optional) - Filter by: `user` or `company`
- `status` (optional) - Filter by: `pending`, `completed`, `failed`, `refunded`
- `event_id` (optional) - Filter by event ID

**Examples:**

- All donations: `GET /donations`
- Company sponsors: `GET /donations?donor_type=company&type=sponsor`
- Event donations: `GET /donations?event_id=5`
- Completed only: `GET /donations?status=completed`

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 123,
      "type": "donate",
      "donor_type": "user",
      "user_id": 5,
      "amount": 50.0,
      "currency": "USD",
      "message": "Happy to support!",
      "status": "completed",
      "transaction_id": "DON-123-1705318200",
      "payment_method": "khqr",
      "event_id": 5,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:32:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 5,
    "total_records": 95,
    "has_next_page": true,
    "has_prev_page": false
  }
}
```

---

### Get Donation by ID

Retrieve a specific donation with full details.

**Endpoint:** `GET /donations/:id`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Success Response (200):**

```json
{
  "id": 123,
  "type": "donate",
  "donor_type": "user",
  "user_id": 5,
  "user": {
    "id": 5,
    "username": "johndoe",
    "fullname": "John Doe",
    "email": "john@example.com"
  },
  "amount": 50.0,
  "currency": "USD",
  "message": "Happy to support!",
  "status": "completed",
  "transaction_id": "DON-123-1705318200",
  "payment_method": "khqr",
  "event_id": 5,
  "event": {
    "id": 5,
    "title": "Charity Concert 2024",
    "location": "Phnom Penh"
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:32:00Z"
}
```

**Error Responses:**

- **404 Not Found:** Donation not found

---

### Get Donations by Type

Retrieve donations filtered by type.

**Endpoint:** `GET /donations/type/:type`  
**Authentication:** None (Public)

**Path Parameters:**

- `type` (required) - `donate` or `sponsor`

**Query Parameters:**

- `limit` (optional, default: 20)
- `offset` (optional, default: 0)

---

### Get Donations by Event

Retrieve all donations for a specific event.

**Endpoint:** `GET /donations/event/:event_id`  
**Authentication:** None (Public)

**Path Parameters:**

- `event_id` (required) - Event ID (integer)

**Query Parameters:**

- `limit` (optional, default: 20)
- `offset` (optional, default: 0)

**Success Response (200):** Returns list of donations for the event

---

### Get User's Donations

Retrieve donations made by the authenticated user.

**Endpoint:** `GET /api/donations/user`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `limit` (optional, default: 20)
- `offset` (optional, default: 0)

**Success Response (200):** Returns list of user's donations

---

### Get Donation Statistics

Get overall donation statistics.

**Endpoint:** `GET /donations/stats`  
**Authentication:** None (Public)

**Success Response (200):**

```json
{
  "total_amount": 15000.0,
  "total_donations": 45,
  "total_sponsors": 12,
  "donate_amount": 8000.0,
  "sponsor_amount": 7000.0,
  "user_donations": 38,
  "company_donations": 19
}
```

---

### Get Event Donation Statistics

Get donation statistics for a specific event.

**Endpoint:** `GET /donations/event/:event_id/stats`  
**Authentication:** None (Public)

**Path Parameters:**

- `event_id` (required) - Event ID (integer)

**Success Response (200):**

```json
{
  "total_amount": 7550.0,
  "total_donations": 15,
  "total_sponsors": 3,
  "donate_amount": 375.0,
  "sponsor_amount": 7175.0,
  "user_donations": 12,
  "company_donations": 6
}
```

---

### Check QR Status

Check if QR code is still valid and payment status.

**Endpoint:** `GET /donations/:id/status`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Success Response (200):**

```json
{
  "donation_id": 123,
  "status": "pending",
  "expired": false
}
```

**Note:** Frontend should poll this endpoint to detect payment completion.

---

### Regenerate QR Code

Generate a new QR code if the previous one expired.

**Endpoint:** `POST /donations/:id/regenerate-qr`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Success Response (200):**

```json
{
  "message": "QR code regenerated successfully",
  "data": {
    "qr_code": "data:image/png;base64,NEW_QR...",
    "expires_at": "2024-01-15T10:36:00Z",
    "expires_in_seconds": 180
  }
}
```

**Error Responses:**

- **400 Bad Request:** Not a QR payment or already completed

---

### Update Donation Status

Update payment status (typically called by payment webhook).

**Endpoint:** `PUT /api/donations/:id/status`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Request Body:**

```json
{
  "status": "completed",
  "transaction_id": "PAYWAY_TXN_123456",
  "payment_method": "aba_pay"
}
```

**Valid Status Values:**

- `pending` - Payment not yet processed
- `completed` - Payment successful
- `failed` - Payment failed
- `refunded` - Payment was refunded

**Success Response (200):**

```json
{
  "message": "donation status updated successfully"
}
```

---

### Delete Donation

Delete a pending donation (only owner can delete).

**Endpoint:** `DELETE /api/donations/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Donation ID (integer)

**Success Response (200):**

```json
{
  "message": "donation deleted successfully"
}
```

**Error Responses:**

- **401 Unauthorized:** User not authenticated
- **400 Bad Request:** Cannot delete completed donations

---

### Payway Webhook (Internal)

Webhook endpoint for Payway payment callbacks.

**Endpoint:** `POST /webhooks/payway`  
**Authentication:** None (Called by Payway)

**Request Body:**

```json
{
  "tran_id": "DON-123-1705318200",
  "status": "success",
  "approval_code": "123456",
  "payment_option": "khqr",
  "hash": "base64_signature"
}
```

**Success Response (200):**

```json
{
  "message": "webhook processed successfully"
}
```

**Note:** This endpoint is automatically called by Payway when payment is completed. It updates the donation status to `completed` or `failed` based on payment result.

---

### Donation Use Cases

#### Use Case 1: General App Support

Donate to support the app (no event_id)

```json
{
  "type": "donate",
  "donor_type": "user",
  "amount": 10.0,
  "currency": "USD",
  "initiate_payment": true
}
```

#### Use Case 2: Event Participation

Pay to join a specific event (with event_id)

```json
{
  "type": "donate",
  "donor_type": "user",
  "amount": 25.0,
  "currency": "USD",
  "event_id": 5,
  "initiate_payment": true
}
```

#### Use Case 3: Company Event Sponsorship

Company sponsors an event with card payment

```json
{
  "type": "sponsor",
  "donor_type": "company",
  "company_name": "Global Bank",
  "company_email": "events@globalbank.com",
  "amount": 5000.0,
  "currency": "USD",
  "event_id": 5,
  "initiate_payment": true
}
```

---

## Supporters

Manage company, organization, and church donor profiles with normalized information and donation tracking.

### Features

- **Three supporter types**: `company`, `organization`, and `church`
- Linked to user accounts for management
- Normalized donor information (eliminates duplicate company data)
- Automatic donation statistics (total donations, total amount)
- Can be used when creating donations instead of inline company info

---

### Create Supporter

Create a new supporter profile (company, organization, or church).

**Endpoint:** `POST /api/supporters`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "name": "Tech Solutions Ltd",
  "email": "contact@techsolutions.com",
  "phone": "+855123456789",
  "type": "company",
  "website": "https://techsolutions.com",
  "address": "123 Business St, Phnom Penh",
  "logo": "https://techsolutions.com/logo.png",
  "description": "Leading IT solutions provider in Cambodia"
}
```

**Validation Rules:**

- `name`: required, min 1 char, max 255 chars
- `email`: required, valid email, max 255 chars, unique
- `phone`: optional, max 50 chars
- `type`: required, `company`, `organization`, or `church`
- `website`: optional, max 255 chars
- `address`: optional, text
- `logo`: optional, max 255 chars (URL)
- `description`: optional, text

**Success Response (201):**

```json
{
  "message": "supporter created successfully",
  "data": {
    "id": 1,
    "name": "Tech Solutions Ltd",
    "email": "contact@techsolutions.com",
    "phone": "+855123456789",
    "type": "company",
    "website": "https://techsolutions.com",
    "address": "123 Business St, Phnom Penh",
    "logo": "https://techsolutions.com/logo.png",
    "description": "Leading IT solutions provider in Cambodia",
    "user_id": 5,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

---

### Get All Supporters

Retrieve all supporters with pagination.

**Endpoint:** `GET /supporters`  
**Authentication:** None (Public)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20, max: 100)

**Success Response (200):** Returns paginated list of supporters

---

### Get Supporter by ID

Retrieve a specific supporter's details.

**Endpoint:** `GET /supporters/:id`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Supporter ID (integer)

**Success Response (200):**

```json
{
  "id": 1,
  "name": "Tech Solutions Ltd",
  "email": "contact@techsolutions.com",
  "phone": "+855123456789",
  "type": "company",
  "website": "https://techsolutions.com",
  "address": "123 Business St, Phnom Penh",
  "logo": "https://techsolutions.com/logo.png",
  "description": "Leading IT solutions provider",
  "user_id": 5,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

---

### Get Supporters by Type

Filter supporters by type (company, organization, or church).

**Endpoint:** `GET /supporters/type/:type`  
**Authentication:** None (Public)

**Path Parameters:**

- `type` (required) - `company`, `organization`, or `church`

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

---

### Get User's Supporters

Retrieve supporters managed by the authenticated user.

**Endpoint:** `GET /api/supporters/user`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

---

### Get Supporter Stats

Get donation statistics for a specific supporter.

**Endpoint:** `GET /api/supporters/:id/stats`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `id` (required) - Supporter ID (integer)

**Success Response (200):**

```json
{
  "id": 1,
  "name": "Tech Solutions Ltd",
  "email": "contact@techsolutions.com",
  "type": "company",
  "total_donations": 15,
  "total_amount": 75000.0,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

---

### Search Supporters by Email

Search for supporters by email address.

**Endpoint:** `GET /supporters/search?email=contact@techsolutions.com`  
**Authentication:** None (Public)

**Query Parameters:**

- `email` (required) - Email address to search

---

### Update Supporter

Update supporter information. Only the managing user can update.

**Endpoint:** `PUT /api/supporters/:id`  
**Authentication:** Required (Bearer Token, must be manager)

**Path Parameters:**

- `id` (required) - Supporter ID (integer)

**Request Body:** Same format as Create Supporter

**Success Response (200):**

```json
{
  "message": "supporter updated successfully",
  "data": {
    /* updated supporter */
  }
}
```

---

### Delete Supporter

Delete a supporter profile. Only the managing user can delete.

**Endpoint:** `DELETE /api/supporters/:id`  
**Authentication:** Required (Bearer Token, must be manager)

**Path Parameters:**

- `id` (required) - Supporter ID (integer)

**Success Response (200):**

```json
{
  "message": "supporter deleted successfully"
}
```

---

## Churches

Manage church profiles with member approval system. Users can join churches and church owners can approve/reject membership requests.

### Features

- **Church Profiles**: Complete information including pastor, denomination, established date
- **Ownership System**: Creator becomes church owner automatically
- **Membership Approval**: Pending → Approved/Rejected workflow
- **Member Management**: View approved members and pending requests
- **User Relationship**: Each user can belong to one church

---

### Create Church

Create a new church profile (user becomes owner).

**Endpoint:** `POST /api/churches`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "fullname": "Grace Baptist Church",
  "address": "456 Church Ave, Phnom Penh",
  "phone": "+855987654331",
  "email": "contact@gracebaptist.org",
  "website": "https://gracebaptist.org",
  "pastor_name": "Rev. John Smith",
  "description": "A vibrant community church committed to worship and service",
  "logo": "https://gracebaptist.org/logo.png",
  "established_date": "1990-05-15",
  "denomination": "Baptist"
}
```

**Validation Rules:**

- `fullname`: required, min 1 char, max 255 chars, unique
- `address`: optional, text
- `phone`: optional, max 50 chars
- `email`: optional, valid email, max 255 chars
- `website`: optional, max 255 chars
- `pastor_name`: optional, max 255 chars
- `description`: optional, text
- `logo`: optional, max 255 chars (URL)
- `established_date`: optional, format: YYYY-MM-DD
- `denomination`: optional, max 100 chars

**Success Response (201):**

```json
{
  "message": "church created successfully",
  "data": {
    "id": 1,
    "fullname": "Grace Baptist Church",
    "address": "456 Church Ave, Phnom Penh",
    "phone": "+855987654331",
    "email": "contact@gracebaptist.org",
    "website": "https://gracebaptist.org",
    "pastor_name": "Rev. John Smith",
    "description": "A vibrant community church",
    "logo": "https://gracebaptist.org/logo.png",
    "established_date": "1990-05-15T00:00:00Z",
    "denomination": "Baptist",
    "owner_id": 5,
    "member_count": 0,
    "pending_count": 0,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

---

### Get All Churches

Retrieve all churches with pagination.

**Endpoint:** `GET /churches`  
**Authentication:** None (Public)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20, max: 100)

**Success Response (200):** Returns paginated list of churches

---

### Get Church by ID

Retrieve a specific church's details including member counts.

**Endpoint:** `GET /churches/:id`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Success Response (200):**

```json
{
  "id": 1,
  "fullname": "Grace Baptist Church",
  "address": "456 Church Ave, Phnom Penh",
  "phone": "+855987654331",
  "email": "contact@gracebaptist.org",
  "website": "https://gracebaptist.org",
  "pastor_name": "Rev. John Smith",
  "description": "A vibrant community church",
  "logo": "https://gracebaptist.org/logo.png",
  "established_date": "1990-05-15T00:00:00Z",
  "denomination": "Baptist",
  "owner_id": 5,
  "owner": {
    "id": 5,
    "username": "johndoe",
    "fullname": "John Doe",
    "email": "john@example.com"
  },
  "member_count": 25,
  "pending_count": 3,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

---

### Get Churches by Denomination

Filter churches by denomination.

**Endpoint:** `GET /churches/denomination/:denomination`  
**Authentication:** None (Public)

**Path Parameters:**

- `denomination` (required) - Denomination name (e.g., Baptist, Catholic, Presbyterian)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

**Success Response (200):** Returns paginated list of churches matching denomination

---

### Get Church Members

Retrieve approved members of a church.

**Endpoint:** `GET /churches/:id/members`  
**Authentication:** None (Public)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 10,
      "username": "janedoe",
      "fullname": "Jane Doe",
      "email": "jane@example.com",
      "profile": "https://example.com/profiles/jane.jpg",
      "church_id": 1,
      "church_status": "approved",
      "created_at": "2024-01-10T10:00:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 2,
    "total_records": 25,
    "has_next_page": true,
    "has_prev_page": false
  }
}
```

---

### Join Church

Request to join a church (status: pending, awaits owner approval).

**Endpoint:** `POST /api/churches/join`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "church_id": 1
}
```

**Success Response (200):**

```json
{
  "message": "church join request submitted. Waiting for owner approval"
}
```

**Error Responses:**

- **400 Bad Request:** User already in a church or church not found
- **401 Unauthorized:** User not authenticated

**Notes:**

- User can only be in one church at a time
- Must leave current church before joining another
- Join request starts with status "pending"

---

### Leave Church

Leave your current church.

**Endpoint:** `POST /api/churches/leave`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "message": "left church successfully"
}
```

**Error Responses:**

- **400 Bad Request:** User not in any church

---

### Get Pending Members (Owner Only)

View pending membership requests for your church.

**Endpoint:** `GET /api/churches/:id/pending`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Query Parameters:**

- `page` (optional)
- `page_size` (optional)

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 15,
      "username": "newuser",
      "fullname": "New User",
      "email": "newuser@example.com",
      "church_id": 1,
      "church_status": "pending",
      "created_at": "2024-01-20T15:30:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 1,
    "total_records": 3,
    "has_next_page": false,
    "has_prev_page": false
  }
}
```

**Error Responses:**

- **400 Bad Request:** Not church owner or church not found

---

### Approve/Reject Member (Owner Only)

Approve or reject a pending membership request.

**Endpoint:** `POST /api/churches/:id/approve`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Request Body:**

```json
{
  "user_id": 15,
  "status": "approved"
}
```

**Valid Status Values:**

- `approved` - Accept the member into the church
- `rejected` - Reject the membership request (removes church_id)

**Success Response (200):**

```json
{
  "message": "member approved successfully"
}
```

Or for rejection:

```json
{
  "message": "member rejected successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Not church owner, user not requesting to join, or invalid status
- **401 Unauthorized:** User not authenticated

---

### Update Church (Owner Only)

Update church information. Only the owner can update.

**Endpoint:** `PUT /api/churches/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Request Body:** Same format as Create Church

**Success Response (200):**

```json
{
  "message": "church updated successfully",
  "data": {
    /* updated church */
  }
}
```

**Error Responses:**

- **400 Bad Request:** Not church owner, church not found, or name already exists
- **401 Unauthorized:** User not authenticated

---

### Delete Church (Owner Only)

Delete a church. Only the owner can delete.

**Endpoint:** `DELETE /api/churches/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Church ID (integer)

**Success Response (200):**

```json
{
  "message": "church deleted successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Not church owner or church not found
- **401 Unauthorized:** User not authenticated

**Notes:**

- Deleting a church removes all member associations
- Members' `church_id` will be set to NULL

---

### Church Membership Workflow

```
1. User creates church → Becomes owner automatically
2. Other users request to join → Status: "pending"
3. Owner views pending requests → GET /api/churches/:id/pending
4. Owner approves/rejects → POST /api/churches/:id/approve
5. If approved → User becomes member (status: "approved")
6. If rejected → User removed from church (church_id = NULL)
7. User can leave anytime → POST /api/churches/leave
```

---

### User Church Information

Users have the following church-related fields:

- `birthday`: User's date of birth (optional)
- `church_id`: ID of church user belongs to (nullable)
- `church_status`: `pending`, `approved`, or `rejected`
- `bio`: User biography/description (optional)

**Example User Response:**

```json
{
  "id": 10,
  "username": "janedoe",
  "fullname": "Jane Doe",
  "email": "jane@example.com",
  "profile": "https://example.com/profiles/jane.jpg",
  "birthday": "1990-05-15T00:00:00Z",
  "church_id": 1,
  "church": {
    "id": 1,
    "fullname": "Grace Baptist Church",
    "pastor_name": "Rev. John Smith"
  },
  "church_status": "approved",
  "bio": "Music ministry leader and worship coordinator",
  "created_at": "2024-01-10T10:00:00Z"
}
```

---

### Donation Status Flow

```
pending → completed  (successful payment)
pending → failed     (failed payment)
completed → refunded (refunded payment)
```

---

### Payment Integration Notes

**Payway by ABA Configuration:**
Required environment variables:

- `PAYWAY_MERCHANT_ID` - Your Payway merchant ID
- `PAYWAY_API_KEY` - Your Payway API key
- `PAYWAY_API_USERNAME` - Your Payway username
- `PAYWAY_BASE_URL` - Sandbox or production URL
- `PAYWAY_RETURN_URL` - Frontend return URL
- `PAYWAY_CALLBACK_URL` - Backend webhook URL

**QR Code Expiration:**

- QR codes expire after 3 minutes for security
- Frontend should show countdown timer
- Users can regenerate expired QR codes
- Card payments have no expiration

**Supported Currencies:**

- `USD` - US Dollar
- `KHR` - Cambodian Riel

---

## Device Tokens (FCM Push Notifications)

Firebase Cloud Messaging (FCM) integration for real-time push notifications to mobile apps and web browsers.

### Features

- Push notifications to iOS, Android, and Web platforms
- Automatic token management and cleanup
- Background notification delivery
- Works seamlessly with the notification system

### Setup Requirements

1. Create a Firebase project at [Firebase Console](https://console.firebase.google.com/)
2. Download the service account key (JSON file)
3. Set environment variable: `FIREBASE_CREDENTIALS_PATH=/path/to/firebase-adminsdk.json`
4. If not set, the API will run with a dummy FCM service (notifications still work, but no push)

---

### Register Device Token

Register a device token to receive push notifications.

**Endpoint:** `POST /api/device-tokens/register`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "token": "fcm_device_token_string_here",
  "platform": "ios"
}
```

**Parameters:**

- `token` (required) - FCM device token from Firebase SDK
- `platform` (required) - Platform type: `ios`, `android`, or `web`

**Success Response (200):**

```json
{
  "message": "Device token registered successfully",
  "token_id": 42
}
```

**Notes:**

- Tokens are automatically upserted (updated if they already exist)
- One user can have multiple device tokens (multiple devices)
- Inactive tokens are automatically cleaned up when FCM reports them as invalid

---

### Unregister Device Token

Remove a device token (e.g., on logout).

**Endpoint:** `POST /api/device-tokens/unregister`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "token": "fcm_device_token_string_here"
}
```

**Success Response (200):**

```json
{
  "message": "Device token unregistered successfully"
}
```

---

### Clear All Device Tokens

Remove all device tokens for the current user.

**Endpoint:** `DELETE /api/device-tokens/clear`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "message": "All device tokens deleted successfully"
}
```

**Use Cases:**

- User logs out from all devices
- Account settings: "Sign out everywhere"
- Privacy/security feature

---

### Mobile Integration Example

#### React Native / Expo

```javascript
import messaging from "@react-native-firebase/messaging";
import axios from "axios";

// Request permission and get token
async function registerForPushNotifications() {
  const authStatus = await messaging().requestPermission();

  if (authStatus === messaging.AuthorizationStatus.AUTHORIZED) {
    const token = await messaging().getToken();

    // Send token to your API
    await axios.post(
      "https://your-api.com/api/device-tokens/register",
      {
        token: token,
        platform: Platform.OS === "ios" ? "ios" : "android",
      },
      {
        headers: { Authorization: `Bearer ${yourJWTToken}` },
      }
    );
  }
}

// Handle foreground notifications
messaging().onMessage(async (remoteMessage) => {
  console.log("Notification received:", remoteMessage);
  // Show in-app notification
});

// Handle notification tap
messaging().onNotificationOpenedApp((remoteMessage) => {
  // Navigate to relevant screen
  if (remoteMessage.data?.related_type === "event") {
    navigation.navigate("EventDetail", {
      id: remoteMessage.data.related_id,
    });
  }
});
```

#### Flutter

```dart
import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:http/http.dart' as http;

Future<void> registerForPushNotifications() async {
  FirebaseMessaging messaging = FirebaseMessaging.instance;

  NotificationSettings settings = await messaging.requestPermission(
    alert: true,
    badge: true,
    sound: true,
  );

  if (settings.authorizationStatus == AuthorizationStatus.authorized) {
    String? token = await messaging.getToken();

    // Send to your API
    await http.post(
      Uri.parse('https://your-api.com/api/device-tokens/register'),
      headers: {
        'Authorization': 'Bearer $yourJWTToken',
        'Content-Type': 'application/json',
      },
      body: jsonEncode({
        'token': token,
        'platform': Platform.isIOS ? 'ios' : 'android',
      }),
    );
  }
}

// Handle foreground messages
FirebaseMessaging.onMessage.listen((RemoteMessage message) {
  // Show notification
});

// Handle notification tap
FirebaseMessaging.onMessageOpenedApp.listen((RemoteMessage message) {
  // Navigate to screen
});
```

#### Web (PWA)

```javascript
import { getMessaging, getToken, onMessage } from "firebase/messaging";

async function registerForPushNotifications() {
  const messaging = getMessaging();

  try {
    const token = await getToken(messaging, {
      vapidKey: "YOUR_VAPID_KEY",
    });

    // Send token to your API
    await fetch("https://your-api.com/api/device-tokens/register", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${yourJWTToken}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        token: token,
        platform: "web",
      }),
    });
  } catch (err) {
    console.error("Error getting FCM token:", err);
  }
}

// Handle foreground messages
onMessage(messaging, (payload) => {
  console.log("Message received:", payload);
  // Show notification
});
```

---

## Settings

### Get User Settings

Retrieve the authenticated user's settings/preferences.

**Endpoint:** `GET /api/settings`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "id": 1,
  "user_id": 5,
  "language": "en",
  "theme": "dark",
  "notify_on_booking": true,
  "notify_on_music": false,
  "notify_on_event": true,
  "enable_push_notifications": true,
  "created_at": "2025-10-25T10:30:00Z",
  "updated_at": "2025-10-29T12:00:00Z"
}
```

**Note:** Settings are automatically created when a user registers via a database trigger.

---

### Update Settings

Update the authenticated user's preferences.

**Endpoint:** `PUT /api/settings`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "language": "kh",
  "theme": "dark",
  "notify_on_booking": true,
  "notify_on_music": true,
  "notify_on_event": true,
  "enable_push_notifications": false
}
```

**Valid Values:**

- `language`: `en` (English), `kh` (Khmer), `kr` (Korean), `cn` (Chinese)
- `theme`: `light`, `dark`, `auto`
- Notification flags: `true` or `false` (boolean)

**Success Response (200):** Returns updated settings object.

**Error Responses:**

- **400 Bad Request:** Invalid settings data or theme not in allowed values
- **401 Unauthorized:** User not authenticated

---

### Reset Settings to Defaults

Reset all user settings to their default values.

**Endpoint:** `POST /api/settings/reset`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "id": 1,
  "user_id": 5,
  "language": "en",
  "theme": "light",
  "notify_on_booking": true,
  "notify_on_music": false,
  "notify_on_event": true,
  "enable_push_notifications": true,
  "created_at": "2025-10-25T10:30:00Z",
  "updated_at": "2025-10-29T12:30:00Z"
}
```

**Default Values:**

- Language: `en`
- Theme: `light`
- Notify on Booking: `true`
- Notify on Music: `false`
- Notify on Event: `true`
- Enable Push Notifications: `true`

---

## Notifications

**Recipient Types:**

- `user` - Send to a specific user
- `band` - Send to all members of a band/team
- `all` - Broadcast to all users in the system

**Features:**

- Users automatically receive notifications sent to them, their band, and all broadcasts
- Read tracking per notification
- Automatic filtering based on user's band membership

---

### Create Notification (To Specific User)

Create a new notification for a specific user.

**Endpoint:** `POST /api/notifications`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "user_id": 10,
  "title": "New Booking Confirmation",
  "message": "Your booking for Sunday Service has been confirmed",
  "type": "booking",
  "related_type": "booking",
  "related_id": 15
}
```

**Validation Rules:**

- `user_id`: required for user notifications
- `title`: required, min 1 char, max 255 chars
- `message`: required, min 1 char
- `type`: required, one of: `info`, `success`, `warning`, `error`, `booking`, `music`, `event`
- `related_type`: optional, one of: `music`, `event`, `booking`, `band`
- `related_id`: optional, integer (ID of related resource)

**Success Response (201):**

```json
{
  "id": 1,
  "user_id": 10,
  "sender_id": 5,
  "recipient_type": "user",
  "title": "New Booking Confirmation",
  "message": "Your booking for Sunday Service has been confirmed",
  "type": "booking",
  "related_type": "booking",
  "related_id": 15,
  "is_read": false,
  "created_at": "2025-10-29T10:30:00Z",
  "updated_at": "2025-10-29T10:30:00Z"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid input or validation error
- **401 Unauthorized:** User not authenticated

---

### Create Band Notification (To Team)

Create a notification for all members of a band/team.

**Endpoint:** `POST /api/notifications/band/:band_id`  
**Authentication:** Required (Bearer Token)

**Path Parameters:**

- `band_id` (required) - Band ID (integer)

**Request Body:**

```json
{
  "title": "New Song Added",
  "message": "Amazing Grace has been added to the band repertoire",
  "type": "music",
  "related_type": "music",
  "related_id": 42
}
```

**Success Response (201):**

```json
{
  "id": 2,
  "band_id": 3,
  "sender_id": 5,
  "recipient_type": "band",
  "title": "New Song Added",
  "message": "Amazing Grace has been added to the band repertoire",
  "type": "music",
  "related_type": "music",
  "related_id": 42,
  "is_read": false,
  "created_at": "2025-10-29T10:35:00Z",
  "updated_at": "2025-10-29T10:35:00Z"
}
```

**Note:** All users who are members of this band (have `band_id` set) will receive this notification.

---

### Create Broadcast Notification (To All Users)

Create a broadcast notification that all users in the system will receive.

**Endpoint:** `POST /api/notifications/broadcast`  
**Authentication:** Required (Bearer Token)

**Request Body:**

```json
{
  "title": "System Maintenance",
  "message": "The system will undergo maintenance on Sunday at 2 AM",
  "type": "warning"
}
```

**Success Response (201):**

```json
{
  "id": 3,
  "sender_id": 1,
  "recipient_type": "all",
  "title": "System Maintenance",
  "message": "The system will undergo maintenance on Sunday at 2 AM",
  "type": "warning",
  "is_read": false,
  "created_at": "2025-10-29T10:40:00Z",
  "updated_at": "2025-10-29T10:40:00Z"
}
```

**Note:** Every user in the system will see this notification. Use sparingly for important announcements.

---

### Get All Notifications

Retrieve all notifications for the authenticated user with pagination.

**Endpoint:** `GET /api/notifications`  
**Authentication:** Required (Bearer Token)

**Query Parameters:**

- `page` (optional, default: 1)
- `page_size` (optional, default: 20, max: 100)

**Example:** `GET /api/notifications?page=1&page_size=20`

**Success Response (200):**

```json
{
  "data": [
    {
      "id": 5,
      "recipient_type": "all",
      "sender_id": 1,
      "title": "System Maintenance",
      "message": "Scheduled maintenance tonight",
      "type": "warning",
      "is_read": false,
      "created_at": "2025-10-29T11:30:00Z",
      "updated_at": "2025-10-29T11:30:00Z"
    },
    {
      "id": 4,
      "band_id": 3,
      "recipient_type": "band",
      "sender_id": 2,
      "title": "Band Practice",
      "message": "Practice rescheduled to Saturday",
      "type": "info",
      "is_read": false,
      "created_at": "2025-10-29T11:15:00Z",
      "updated_at": "2025-10-29T11:15:00Z"
    },
    {
      "id": 3,
      "user_id": 5,
      "recipient_type": "user",
      "sender_id": 2,
      "title": "New Music Added",
      "message": "Amazing Grace has been added to your band",
      "type": "music",
      "related_type": "music",
      "related_id": 12,
      "is_read": false,
      "created_at": "2025-10-29T11:00:00Z",
      "updated_at": "2025-10-29T11:00:00Z"
    },
    {
      "id": 2,
      "user_id": 5,
      "recipient_type": "user",
      "title": "Event Reminder",
      "message": "Sunday Service starts in 1 hour",
      "type": "event",
      "related_type": "event",
      "related_id": 8,
      "is_read": true,
      "read_at": "2025-10-29T10:45:00Z",
      "created_at": "2025-10-29T10:30:00Z",
      "updated_at": "2025-10-29T10:45:00Z"
    }
  ],
  "pagination": {
    "current_page": 1,
    "page_size": 20,
    "total_pages": 1,
    "total_records": 2,
    "has_next_page": false,
    "has_prev_page": false
  }
}
```

---

### Get Unread Notifications

Retrieve all unread notifications for the authenticated user.

**Endpoint:** `GET /api/notifications/unread`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
[
  {
    "id": 3,
    "user_id": 5,
    "title": "New Music Added",
    "message": "Amazing Grace has been added to your band",
    "type": "music",
    "related_type": "music",
    "related_id": 12,
    "is_read": false,
    "created_at": "2025-10-29T11:00:00Z",
    "updated_at": "2025-10-29T11:00:00Z"
  }
]
```

---

### Get Unread Count

Get the count of unread notifications for the authenticated user.

**Endpoint:** `GET /api/notifications/unread/count`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "count": 5
}
```

**Note:** This is useful for displaying notification badges in the UI.

---

### Get Notification by ID

Retrieve a specific notification.

**Endpoint:** `GET /api/notifications/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Notification ID (integer)

**Success Response (200):**

```json
{
  "id": 3,
  "user_id": 5,
  "title": "New Music Added",
  "message": "Amazing Grace has been added to your band",
  "type": "music",
  "related_type": "music",
  "related_id": 12,
  "is_read": false,
  "created_at": "2025-10-29T11:00:00Z",
  "updated_at": "2025-10-29T11:00:00Z"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid ID format
- **404 Not Found:** Notification not found
- **401 Unauthorized:** User doesn't own this notification

---

### Mark Notification as Read

Mark a specific notification as read.

**Endpoint:** `PUT /api/notifications/:id/read`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Notification ID (integer)

**Success Response (200):**

```json
{
  "message": "notification marked as read"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid ID or operation failed
- **401 Unauthorized:** User doesn't own this notification
- **404 Not Found:** Notification not found

---

### Mark All Notifications as Read

Mark all notifications as read for the authenticated user.

**Endpoint:** `PUT /api/notifications/read-all`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "message": "all notifications marked as read"
}
```

**Note:** This operation affects all unread notifications for the user.

---

### Delete Notification

Delete a specific notification.

**Endpoint:** `DELETE /api/notifications/:id`  
**Authentication:** Required (Bearer Token, must be owner)

**Path Parameters:**

- `id` (required) - Notification ID (integer)

**Success Response (200):**

```json
{
  "message": "notification deleted successfully"
}
```

**Error Responses:**

- **400 Bad Request:** Invalid ID or operation failed
- **401 Unauthorized:** User doesn't own this notification
- **404 Not Found:** Notification not found

---

### Delete All Notifications

Delete all notifications for the authenticated user.

**Endpoint:** `DELETE /api/notifications`  
**Authentication:** Required (Bearer Token)

**Success Response (200):**

```json
{
  "message": "all notifications deleted successfully"
}
```

**Note:** This permanently deletes all notifications for the user. This action cannot be undone.

---

## Health Check

### Ping

Simple endpoint to check if the API is running and responsive.

**Endpoint:** `GET /ping`  
**Authentication:** None (Public)

**Success Response (200):**

```json
{
  "message": "api work..."
}
```

**Usage:** Use this for health checks, monitoring, load balancer health checks, etc.

---

## Common HTTP Status Codes

### Success Codes

- **200 OK** - Request succeeded
- **201 Created** - Resource created successfully

### Client Error Codes

- **400 Bad Request** - Invalid input, validation error, or business logic error
- **401 Unauthorized** - Missing or invalid authentication token
- **403 Forbidden** - User doesn't have permission (e.g., not the owner)
- **404 Not Found** - Resource not found

### Server Error Codes

- **500 Internal Server Error** - Unexpected server error

---

## Error Response Format

All error responses follow this format:

```json
{
  "error": "error message here"
}
```

Or with validation errors:

```json
{
  "errors": "Username is required"
}
```

---

## Authentication Flow

### Standard Flow (Email/Password)

1. **Register** → `POST /register` → Receive JWT token
2. **Login** → `POST /login` → Receive JWT token
3. **Use Token** → Include in `Authorization: Bearer <token>` header for all protected routes
4. **Logout** → `POST /api/logout` → Token invalidated

### OAuth Flow (Google)

1. **Get Google ID Token** → From Google OAuth flow on client side
2. **Authenticate** → `POST /auth/google` with ID token → Receive JWT token
3. **Use Token** → Same as standard flow

### Token Usage

Include the JWT token in the `Authorization` header for all protected routes:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Pagination

### Request

Most list endpoints support pagination via query parameters:

```
GET /api/musics?page=2&page_size=25
```

- `page`: Page number (default: 1)
- `page_size`: Items per page (default: 20, max: 100)

### Response

Paginated responses include metadata:

```json
{
  "data": [
    /* array of items */
  ],
  "pagination": {
    "current_page": 2,
    "page_size": 25,
    "total_pages": 10,
    "total_records": 245,
    "has_next_page": true,
    "has_prev_page": true
  }
}
```

---

## Database Schema Notes

### Tables

- **roles** - System roles with permissions (JSON)
- **users** - User accounts with authentication data (includes birthday, church_id, church_status, bio)
- **churches** - Church profiles with owner management and member approval system
- **tokens** - Multi-device token management
- **settings** - User preferences (auto-created on registration)
- **musics** - Core music metadata
- **music_audio** - Multiple audio files per music (Original, Instrumental, etc.)
- **music_sheets** - Sheet music files (Lead Sheet, Chord Chart, etc.)
- **events** - Event scheduling
- **event_musics** - Many-to-many: Events ↔ Music
- **bookings** - Event registrations
- **bands** - Music collections/libraries
- **band_musics** - Many-to-many: Bands ↔ Music
- **favorites** - User favorites (Many-to-many: Users ↔ Music)
- **donations** - Donations and sponsorships with payment tracking
- **supporters** - Normalized company/organization/church donor profiles
- **notifications** - User notifications with read tracking
- **device_tokens** - FCM push notification tokens

### Automatic Features

- **Timestamps** - All tables have `created_at` and `updated_at` with auto-update triggers
- **Cascade Deletes** - Deleting a user/music/event automatically deletes related records
- **Settings Creation** - New users automatically get default settings via trigger
- **Timezone Support** - All timestamps use `TIMESTAMPTZ` (UTC, converted to Asia/Phnom_Penh)

---

## Environment Variables

```bash
# Server Configuration
PORT=8080

# Database
POSTGRES_URL=postgres://user:password@host:5433/dbname?sslmode=disable

# Security
SECRET=your_jwt_secret_key_here

# OAuth (Optional)
GOOGLE_CLIENT_ID=your_google_client_id_here

# Payway by ABA (Payment Gateway)
PAYWAY_MERCHANT_ID=your_merchant_id
PAYWAY_API_KEY=your_api_key
PAYWAY_API_USERNAME=your_api_username
PAYWAY_BASE_URL=https://api-sandbox.payway.com.kh  # or production URL
PAYWAY_RETURN_URL=http://localhost:3000/donation/complete
PAYWAY_CONTINUE_URL=http://localhost:3000/donation/success
PAYWAY_CALLBACK_URL=http://localhost:8080/webhooks/payway

# Firebase (Optional - for push notifications)
FIREBASE_CREDENTIALS_PATH=/path/to/firebase-adminsdk.json

# Gin Mode
GIN_MODE=release  # or 'debug' for development
```

---

## Notes

- All authenticated endpoints require a valid JWT token in the `Authorization` header
- All timestamps are in ISO 8601 format with timezone (UTC)
- The API uses UTC internally but converts to `Asia/Phnom_Penh` timezone
- Users can only modify/delete their own resources (music, events, bookings, bands)
- All POST/PUT requests should include `Content-Type: application/json` header
- Music can belong to multiple bands and events (many-to-many relationships)
- Settings are automatically created when a user registers via database trigger
- Audio files are stored in the `music_audio` table (supports multiple files per music: Original, Instrumental, Acapella, Live, Acoustic, Remix, Cover)
- Sheet music files are stored in the `music_sheets` table (supports multiple sheets per music with different types and languages)
- Music metadata includes: artist, album, genre, duration, BPM, key, cover, lyrics, and description
- Notifications support multiple types (info, success, warning, error, booking, music, event) and can be linked to related resources
- Notifications track read status and read timestamps for user experience
- Notifications can be sent to specific users, bands/teams, or broadcast to all users
- Users automatically see notifications sent directly to them, to their band, and all broadcast notifications
- Sender tracking to know who created each notification
- Users do not receive notifications they created themselves (self-notifications are filtered out)
- Creating an event automatically sends a broadcast notification to all users (except the event creator)
- Firebase Cloud Messaging (FCM) is integrated for push notifications to mobile and web
- Device tokens are stored in the `device_tokens` table and support iOS, Android, and Web platforms
- Push notifications are sent automatically when notifications are created (works in background)
- Invalid device tokens are automatically cleaned up when FCM reports them
- Donations support two types: `donate` (QR payment) and `sponsor` (card payment)
- Donations can be for app support (no event_id) or event participation (with event_id)
- QR codes expire after 3 minutes for security and can be regenerated
- Payment integration with Payway by ABA for both QR (KHQR) and card payments
- Donation statistics available globally and per-event
- Companies/organizations/churches can donate without authentication or use supporter profiles
- Users require login for donations
- Supporters normalize donor information (companies, organizations, churches)
- Churches have ownership and member approval system
- Users can join one church at a time with pending/approved/rejected status
- Church owners can approve/reject membership requests
- Users have additional fields: birthday, church_id, church_status, bio

---

**Last Updated:** October 30, 2025  
**API Version:** 1.0  
**Database:** PostgreSQL 16  
**Framework:** Gin (Go)
