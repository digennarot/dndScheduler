use redb::TableDefinition;

// POLLS
// Key: poll_id (String)
// Value: Poll (JSON bytes)
pub const POLLS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("polls");

// POLL_INSTANCES
// Key: instance_id (String)
// Value: PollInstance (JSON bytes)
pub const POLL_INSTANCES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("poll_instances");

// PARTICIPANTS
// Key: "{poll_id}:{participant_id}" (String)
// Value: Participant (JSON bytes)
pub const PARTICIPANTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("participants");

// AVAILABILITY
// Key: "{poll_id}:{participant_id}:{date}:{time_slot}" (String)
// Value: Availability (JSON bytes)
pub const AVAILABILITY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("availability");

// USERS
// Key: user_id (String)
// Value: User (JSON bytes)
pub const USERS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("users");

// USERS_BY_EMAIL (Secondary Index)
// Key: email (String)
// Value: user_id (String)
pub const USERS_BY_EMAIL_TABLE: TableDefinition<&str, &str> = TableDefinition::new("users_by_email");

// USER_SESSIONS
// Key: token (String)
// Value: UserSession (JSON bytes)
pub const USER_SESSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("user_sessions");

// USER_SESSIONS_BY_USER_ID (Index)
// Key: user_id (String)
// Value: token (String)
pub const USER_SESSIONS_BY_USER_ID_TABLE: TableDefinition<&str, &str> = TableDefinition::new("user_sessions_by_user_id");

// ADMINS
// Key: admin_id (String)
// Value: Admin (JSON bytes)
pub const ADMINS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("admins");

// ADMINS_BY_EMAIL (Index)
// Key: email (String)
// Value: admin_id (String)
pub const ADMINS_BY_EMAIL_TABLE: TableDefinition<&str, &str> = TableDefinition::new("admins_by_email");

// ADMINS_BY_USERNAME (Index)
// Key: username (String)
// Value: admin_id (String)
pub const ADMINS_BY_USERNAME_TABLE: TableDefinition<&str, &str> = TableDefinition::new("admins_by_username");

// ADMIN_SESSIONS
// Key: token (String)
// Value: { "user_id": "<ID>", "expires_at": <TIMESTAMP> } (JSON bytes)
pub const ADMIN_SESSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");

// ACTIVITIES
// Key: "{timestamp}:{id}" (String) -> ordered naturally by time
// Value: Activity (JSON bytes)
pub const ACTIVITIES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("activities");

// OWASP: LOGIN ATTEMPTS
// Key: "{email}:{timestamp}:{id}" (String)
// Value: LoginAttempt (JSON bytes)
pub const LOGIN_ATTEMPTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("login_attempts");

// OWASP: ACCOUNT LOCKS
// Key: email (String)
// Value: AccountLock (JSON bytes)
pub const ACCOUNT_LOCKS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("account_locks");

// OWASP: AUDIT LOGS
// Key: "{timestamp}:{id}" (String)
// Value: AuditLog (JSON bytes)
pub const AUDIT_LOG_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("audit_log");

// GDPR: CONSENT RECORDS
// Key: "{user_id}:{timestamp}:{id}" (String)
// Value: ConsentRecord (JSON bytes)
pub const CONSENT_RECORDS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("consent_records");

// GDPR: DATA EXPORT REQUESTS
// Key: request_id (String)
// Value: DataExportRequest (JSON bytes)
pub const DATA_EXPORT_REQUESTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("data_export_requests");
