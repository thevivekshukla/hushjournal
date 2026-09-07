e2ejournal is end-to-end encrypted journal web app. It encrypts user content using AES-256-GCM-SIV on the client side, server side only stores the encrypted content and can't ever see the plain content.

User can log in using Google oauth2, they can create multiple "Spaces", each space will have entries. An entry is title, tags, content (encrypted). By default when they create new entry its title will have prefilled title as today's date such as "7 Sep 2026".

## Signup / Login

- Google oauth will be used for login.
- If there is no user account with email id then it will create new account.

## Dashboard

- Each user can have multiple "Spaces", which is kind of a workspace
- A Space will have multiple entries
- An Entry consists of title, tags and content
- content of the Entry will be encrypted using AES-256-GCM-SIV on the client side, backend will never see the users' content