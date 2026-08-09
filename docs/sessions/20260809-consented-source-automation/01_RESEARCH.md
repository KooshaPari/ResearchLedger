# Research

- LinkedIn's User Agreement prohibits scraping/copying service data and unauthorized bots:
  [agreement](https://www.linkedin.com/legal/user-agreement).
- LinkedIn member APIs require member approval and appropriate permissions:
  [OAuth overview](https://learn.microsoft.com/en-us/linkedin/shared/authentication/authentication).
- GitHub documents OAuth Device Flow, JSON responses, polling interval, cancellation, and
  expiration: [GitHub OAuth authorization](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/authorizing-oauth-apps).
- `GET /user/starred` is documented as the authenticated-user endpoint; it requires the
  appropriate read permission and supports pagination: [GitHub starring API](https://docs.github.com/en/rest/activity/starring?apiVersion=2022-11-28).

Prior local diagnosis showed LinkedIn loading/HTTP 999 and no verified GitHub device grant;
those are capability/auth failures, not permission to introduce scraping or credential capture.
