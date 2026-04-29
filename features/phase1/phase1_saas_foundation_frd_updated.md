# Feature Requirement Document (FRD)
## Phase 1 — SaaS Foundation
## AI-Driven Project Management System

**Date:** 2026-04-29

---

## 1. Purpose

This document defines the Phase 1 requirements for transforming the Phase 0 initial system into a SaaS-ready platform.

In Phase 0, the system supports simple individual user registration and project management.

In Phase 1, the system introduces:

- Company-based accounts
- Multiple users under a company
- Personal accounts that continue to work like Phase 0
- Company admin onboarding
- Team member invitation flow
- System-wide administrator access
- Role-based access control
- Secure AI configuration
- Audit logging
- Usage tracking

The goal is to support both individual users and companies while preparing the platform for cloud hosting and multi-company usage.

---

## 2. Scope

Phase 1 includes:

- Company registration
- Personal registration
- Company user management
- Company admin role
- Email-based team invitations
- System-wide admin user
- Admin dashboard tools
- Company-level AI settings
- User roles and permissions
- Data isolation
- Audit logging
- AI usage tracking

Phase 1 does not include:

- Billing
- SSO
- OAuth/social login
- Advanced organization hierarchies
- Public marketplace features
- External integrations such as Jira, GitHub, or Linear

---

## 3. Terminology

### 3.1 Company

A Company represents an organization using the system.

A company may have:

- Multiple users
- Multiple projects
- Uploaded project documents
- Company-level AI settings
- Company-level usage records
- Company-level audit logs

All company data must be isolated from other companies.

### 3.2 Personal Account

A Personal Account represents an individual user who uses the system without creating a company.

This is the recommended name instead of simply "Person".

Reason:

- "Person" is too generic
- "Personal Account" clearly means individual usage
- It creates a clean distinction from "Company Account"

Recommended UI:

```text
Choose account type:
[ Personal Account ]
[ Company Account ]
```

### 3.3 Company Account

A Company Account represents a registered company workspace.

When a user chooses Company Account during registration, they must create:

- Company profile
- First company user account

The first user account becomes the Company Admin by default.

### 3.4 System Admin

A System Admin is a platform-wide administrator.

This admin is not bound to a company.

The System Admin can access administration tools for managing the entire SaaS platform.

---

## 4. Registration Requirements

### 4.1 Account Type Selection

During registration, the user must first choose one of the following account types:

```text
1. Personal Account
2. Company Account
```

### 4.2 Personal Account Registration

If the user selects Personal Account:

- Registration continues like Phase 0
- User creates a simple account
- User can create multiple projects
- User does not belong to a company
- User cannot invite team members in Phase 1 unless later converted to a Company Account

Required fields:

- Name
- Email
- Username
- Password
- Password confirmation

Optional fields:

- Display name
- Timezone

### 4.3 Company Account Registration

If the user selects Company Account:

The registration flow must continue with company setup.

The user must provide company information and create the first user account.

The created user becomes Company Admin by default.

### 4.4 Company Registration Fields

Required company fields:

- Company name
- Company email
- Country
- Timezone

Recommended optional company fields:

- Legal company name
- Website
- Industry
- Company size
- Phone number
- Billing email
- Address
- Tax/VAT number
- Default AI provider preference
- Data retention preference

Notes:

- Billing fields may be collected later if billing is not part of Phase 1.
- Tax/VAT number should remain optional in Phase 1.
- Company email may be used for administrative notifications.

### 4.5 Company Admin User Fields

The company admin user must provide:

- Full name
- Email
- Username
- Password
- Password confirmation

After registration:

- A Company record is created
- A Company Admin user is created
- The user is linked to the company
- The user role is set to Company Admin
- The user is redirected to the company dashboard

---

## 5. User Model Requirements

### 5.1 User Account Types

The system must support these user account categories:

```text
System Admin
Personal User
Company User
```

### 5.2 Company User Roles

The system must support the following company roles:

```text
Company Admin
Project Manager
Contributor
Viewer
```

Optional future role:

```text
Company Owner
```

For Phase 1, Company Admin is enough as the highest company-level role.

### 5.3 Role Permissions

| Action | Company Admin | Project Manager | Contributor | Viewer |
|---|---:|---:|---:|---:|
| Manage company profile | Yes | No | No | No |
| Invite users | Yes | Yes | No | No |
| Manage users | Yes | No | No | No |
| Create projects | Yes | Yes | Yes | No |
| Edit project requirements | Yes | Yes | Yes | No |
| Create features | Yes | Yes | Yes | No |
| Edit feature requirements | Yes | Yes | Yes | No |
| Create tasks manually | Yes | Yes | Yes | No |
| Generate tasks using AI | Yes | Yes | Yes | No |
| Upload project documents | Yes | Yes | Yes | No |
| Select documents for AI context | Yes | Yes | Yes | No |
| View company AI settings | Yes | No | No | No |
| Edit company AI settings | Yes | No | No | No |
| Delete projects | Yes | No | No | No |
| View projects | Yes | Yes | Yes | Yes |

---

## 6. Team Invitation Requirements

### 6.1 Invite Team Members

Company Admin and optionally Project Manager users must be able to invite team members by email.

Invitation form fields:

- Email address
- Role
- Optional project assignment
- Optional message

If a project is selected, the invited user must be bound to that project after registration.

If no project is selected, the invited user becomes a company-level user without project assignment.

### 6.2 Invitation Email

The system must send an invitation email to the invited team member.

The email must include:

- Company name
- Inviter name
- Invited role
- Project name if applicable
- Invitation link
- Expiration information

### 6.3 Invitation Link

The invitation link must contain a unique invitation identifier.

Example:

```text
https://app.example.com/invitations/accept?id=INVITATION_ID
```

The user requested that the invitation link contain an ID number. For security, the implementation should use a long random token or UUID rather than a predictable numeric ID.

Recommended:

```text
invitation_token = cryptographically secure UUID/token
```

The system must store this invitation identifier and validate it when the invited user opens the invitation page.

### 6.4 Invitation Record

The system must store invitation records.

Fields:

- id
- invitation_token
- company_id
- project_id
- invited_email
- invited_role
- invited_by_user_id
- status
- expires_at
- accepted_at
- created_at

Allowed invitation statuses:

```text
Pending
Accepted
Expired
Cancelled
```

### 6.5 Invitation Acceptance

When the invited user opens the invitation link:

- System validates the invitation token
- System checks expiration
- System checks invitation status
- System displays registration page
- User creates an account
- User is linked to the company
- User receives the invited role
- If project_id exists, user is bound to the specified project
- Invitation status becomes Accepted

### 6.6 Existing User Invitation

If the invited email already belongs to an existing user:

- System must detect the existing user
- System must not create duplicate users
- System must link the existing user to the company/project if allowed
- System must require login before accepting the invitation

For Phase 1, a user may belong to only one company unless multi-company membership is explicitly added later.

---

## 7. Company Data Isolation Requirements

### 7.1 Isolation Rule

Users must only access data that belongs to:

- Their personal account, or
- Their company

Company users must not access another company's data.

Personal users must not access company data unless they are invited and converted/linked according to system rules.

### 7.2 Query Enforcement

All company-owned entities must include:

```text
company_id
```

All personal-owned entities must include:

```text
owner_user_id
```

The system must enforce data access through backend authorization checks.

Frontend hiding is not enough.

### 7.3 Company-Owned Entities

The following entities must be company-scoped when used under a company account:

- Projects
- Features
- Tasks
- Documents
- AI settings
- AI usage records
- Audit logs
- Invitations
- Project memberships

---

## 8. System Admin Requirements

### 8.1 System Admin Credentials

The system must support a system-wide admin user.

For Phase 1, the admin credentials will be stored in the `.env` file.

Required `.env` fields:

```text
SYSTEM_ADMIN_EMAIL=
SYSTEM_ADMIN_PASSWORD_HASH=
```

Recommended:

```text
SYSTEM_ADMIN_ENABLED=true
```

Important security note:

- Store a password hash, not a plain password.
- Never commit `.env` files to source control.
- System Admin authentication attempts must be logged.

### 8.2 System Admin Login

The System Admin may use the same login screen as regular users.

The system must detect System Admin login by checking the submitted email against configured system admin identity.

Suggested approach:

1. User enters email and password in normal login form
2. System checks whether email matches `SYSTEM_ADMIN_EMAIL`
3. If yes, verify password against `SYSTEM_ADMIN_PASSWORD_HASH`
4. If valid, create a System Admin session
5. Redirect to System Admin Dashboard

### 8.3 Alternative Better Approach

A better long-term approach is to create a dedicated admin route:

```text
/admin/login
```

Benefits:

- Clear separation from normal user login
- Easier to apply stricter security
- Easier to add MFA later
- Easier to monitor admin login attempts

Recommended Phase 1 decision:

- Use the same login screen for simplicity
- Internally separate System Admin session type from normal user session
- Prepare architecture so `/admin/login` can be added later

### 8.4 System Admin Dashboard

When System Admin logs in, the dashboard must display admin tools instead of normal user project tools.

System Admin dashboard should include:

#### Company Management

- View all companies
- Search companies
- View company details
- Enable/disable company
- View company users
- View company project count
- View company document count
- View company AI usage summary

#### User Management

- View all users
- Search users by email/name/company
- View user role
- View account type
- Enable/disable user
- Reset user access if required

#### AI Usage Monitoring

- View AI usage by company
- View AI usage by user
- View token usage
- View estimated cost
- View failed AI requests
- View high-usage companies

#### Audit Log Viewer

- View system-level audit logs
- Filter by company
- Filter by user
- Filter by action type
- Filter by date range

#### Invitation Monitoring

- View pending invitations
- View expired invitations
- Cancel suspicious invitations

#### System Health

- View application status
- View background job status
- View AI provider error rate
- View email delivery status
- View storage usage

#### Configuration Tools

- Manage allowed AI providers
- Manage global usage limits
- Manage platform announcement message
- Manage feature flags

---

## 9. AI Settings Requirements

### 9.1 Company-Level AI Settings

Company accounts must store AI settings at company level.

Fields:

- company_id
- provider
- encrypted_api_key
- allowed_models
- monthly_token_limit
- usage_tracking_enabled

### 9.2 Personal Account AI Settings

Personal accounts may store AI settings at user level.

Fields:

- user_id
- provider
- encrypted_api_key
- allowed_models
- monthly_token_limit
- usage_tracking_enabled

### 9.3 AI API Key Security

AI API keys must:

- Be encrypted at rest
- Never be stored as plain text
- Never be returned in full to the frontend
- Never be written to logs
- Be editable only by authorized users

For display, the frontend may show masked values:

```text
sk-****abcd
```

---

## 10. Audit Logging Requirements

The system must log critical actions.

### 10.1 Company Audit Events

Events to log:

- Company registration
- Company profile update
- User invitation created
- Invitation accepted
- Invitation cancelled
- User role changed
- User disabled/enabled
- Project created/updated/deleted
- Feature created/updated/deleted
- Task created/updated/deleted
- Document uploaded/deleted
- AI settings changed
- AI task generation requested
- AI requirement enhancement requested

### 10.2 System Admin Audit Events

Events to log:

- System Admin login success
- System Admin login failure
- Company disabled/enabled
- User disabled/enabled
- Global configuration changed
- Suspicious usage reviewed

### 10.3 Audit Log Fields

Audit log fields:

- id
- company_id nullable
- user_id nullable
- system_admin_email nullable
- action_type
- entity_type
- entity_id
- metadata
- ip_address
- user_agent
- created_at

---

## 11. AI Usage Tracking Requirements

The system must track AI usage for both Personal Accounts and Company Accounts.

Fields:

- id
- company_id nullable
- user_id
- project_id nullable
- feature_id nullable
- operation_type
- provider
- model
- input_tokens
- output_tokens
- estimated_cost
- status
- error_code nullable
- created_at

Operation types:

- Enhance Project Requirements
- Split Project Requirements into Features
- Enhance Feature Requirements
- Generate Tasks
- Regenerate Tasks

---

## 12. Updated Data Model

### 12.1 Company

Fields:

- id
- name
- legal_name nullable
- company_email
- website nullable
- industry nullable
- company_size nullable
- country
- timezone
- phone nullable
- billing_email nullable
- address nullable
- tax_vat_number nullable
- status
- created_at
- updated_at

Statuses:

```text
Active
Disabled
PendingVerification
```

### 12.2 User

Fields:

- id
- account_type
- company_id nullable
- full_name
- display_name nullable
- email
- username
- password_hash
- role nullable
- status
- timezone nullable
- created_at
- updated_at

Account types:

```text
Personal
Company
SystemAdminSessionOnly
```

User statuses:

```text
Active
Disabled
PendingInvitation
```

### 12.3 Project

Fields:

- id
- company_id nullable
- owner_user_id nullable
- name
- requirements_rich_text
- status
- created_by_user_id
- created_at
- updated_at

Status values:

```text
Pending
In Progress
Done
```

### 12.4 Project Membership

Fields:

- id
- project_id
- user_id
- role
- created_at

Purpose:

- Allows project-level access control inside a company
- Supports invited users being bound to a specific project

### 12.5 Invitation

Fields:

- id
- invitation_token
- company_id
- project_id nullable
- invited_email
- invited_role
- invited_by_user_id
- status
- expires_at
- accepted_at nullable
- created_at

### 12.6 AI Settings

Fields:

- id
- company_id nullable
- user_id nullable
- provider
- encrypted_api_key
- allowed_models
- monthly_token_limit
- usage_tracking_enabled
- created_at
- updated_at

Constraint:

- Either company_id or user_id must be set.
- Both must not be set at the same time.

### 12.7 Audit Log

Fields:

- id
- company_id nullable
- user_id nullable
- system_admin_email nullable
- action_type
- entity_type
- entity_id nullable
- metadata
- ip_address nullable
- user_agent nullable
- created_at

### 12.8 AI Usage

Fields:

- id
- company_id nullable
- user_id
- project_id nullable
- feature_id nullable
- operation_type
- provider
- model
- input_tokens
- output_tokens
- estimated_cost
- status
- error_code nullable
- created_at

---

## 13. Security Requirements

### 13.1 Password Security

- Passwords must be hashed using Argon2 or bcrypt
- Plain passwords must never be stored
- Passwords must never be logged

### 13.2 Session Security

- System Admin sessions must be distinguishable from regular user sessions
- Company user sessions must contain company_id
- Personal user sessions must contain user_id
- Authorization must be checked on every protected backend request

### 13.3 Invitation Security

- Invitation tokens must be hard to guess
- Invitation links must expire
- Invitation tokens must be single-use
- Expired or accepted invitation links must not be reusable

### 13.4 Company Isolation

- Company data must never be exposed to users from another company
- Backend must enforce company_id filtering
- System Admin access must be explicitly separated from company user access

---

## 14. Non-Functional Requirements

### 14.1 Scalability

The system should support:

- Many companies
- Many users per company
- Many projects per company
- High AI usage volume
- Large document uploads in later phases

### 14.2 Reliability

The system must:

- Preserve company data isolation
- Keep audit logs durable
- Prevent invitation misuse
- Track failed AI usage attempts

### 14.3 Maintainability

The system should be designed with clear separation between:

- Personal account logic
- Company account logic
- System Admin logic
- AI configuration logic
- Invitation logic
- Authorization logic

---

## 15. Acceptance Criteria

### 15.1 Registration

- User can select Personal Account or Company Account
- Personal Account registration works like Phase 0
- Company Account registration creates company and first Company Admin user
- Company Admin is redirected to company dashboard

### 15.2 Invitation

- Company Admin can invite users by email
- Invitation email contains valid invitation link
- Invitation link opens registration page
- User registration through invitation binds user to correct company
- If project is specified, user is bound to correct project
- Invitation cannot be reused after acceptance

### 15.3 System Admin

- System Admin can login using configured credentials
- System identifies System Admin separately from normal users
- System Admin sees admin dashboard
- System Admin can view companies, users, AI usage, audit logs, and system health

### 15.4 Security

- Passwords are stored securely
- AI API keys are encrypted
- Users cannot access another company's data
- Personal users cannot access company data unless invited/linked
- Company users cannot access system admin tools

### 15.5 Audit and Usage

- Critical actions are recorded in audit logs
- AI usage is tracked per user and per company
- System Admin actions are logged

---

## 16. Constraints

- No billing in Phase 1
- No SSO in Phase 1
- No OAuth/social login in Phase 1
- System Admin credentials are stored in `.env` for Phase 1
- Email delivery provider must be configured separately
- A user may belong to only one company in Phase 1 unless extended later

---

## 17. Future Enhancements

Potential future improvements:

- Dedicated `/admin/login` route
- Multi-factor authentication for System Admin
- SSO for companies
- Billing and subscriptions
- Company verification flow
- Multi-company membership for consultants
- Advanced project-level permissions
- Global feature flags
- Admin impersonation with strict audit controls
- Organization departments/teams
- Custom company roles
- Custom workflow statuses
