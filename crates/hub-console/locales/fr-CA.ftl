## Shell
app-name = Levix Admin Hub
skip-to-content = Aller au contenu
loading = Chargement…
language = Langue
dismiss = Fermer
sign-out = Se déconnecter
nav-label = Principal
nav-organizations = Organisations
nav-audit = Journal d’audit
nav-profile = Profil
nav-sessions = Sessions
nav-tokens = Jetons d’API
nav-instance = Instance
save = Enregistrer
saved = Enregistré.

## Notices
notice-session-ended = Votre session a pris fin. Connectez-vous de nouveau.
notice-signed-out = Vous êtes déconnecté.
notice-inactivity = Vous avez été déconnecté après une période d’inactivité.
notice-reenroll = Votre authentificateur a été réinitialisé. Configurez-le de nouveau pour vous connecter.

## Inactivity
idle-title = Toujours là?
idle-text = Vous serez déconnecté dans une minute pour cause d’inactivité.
idle-stay = Rester connecté

## Not initialized and not found
not-initialized-title = Ce Hub n’est pas encore initialisé
not-initialized-text = Initialisez-le une fois sur l’hôte où il s’exécute :
not-initialized-next = La commande affiche l’empreinte du certificat et un lien de configuration pour le propriétaire de l’instance. Ouvrez ce lien ici.
not-found-title = Page introuvable
not-found-home = Aller à la page d’accueil

## Sign-in
login-title = Se connecter
login-submit = Se connecter
field-email = Courriel
field-password = Mot de passe
field-code = Code de l’authentificateur
field-code-help = Le code à 6 chiffres de votre application d’authentification, ou l’un de vos codes de récupération.

## Links
link-title = Configurer votre compte
link-intro = Ce lien de { $purpose } est destiné à { $email } et est valide jusqu’au { $expires }.
purpose-setup = configuration
purpose-invitation = invitation
purpose-recovery = récupération
enroll-intro = D’abord, ajoutez le Hub à votre application d’authentification.
enroll-start = Configurer l’authentificateur
enroll-title = Authentificateur
enroll-scan = Balayez ce code avec votre application d’authentification.
enroll-qr-alt = Code QR du secret de l’authentificateur
enroll-manual = Ou saisissez ce secret à la main :
field-display-name = Nom affiché
field-new-password = Nouveau mot de passe
field-password-help = Au moins 12 caractères. Les mots de passe courants sont refusés.
field-confirm-password = Répétez le nouveau mot de passe
link-submit = Enregistrer et se connecter
link-deliver-help = Remettez ce lien à usage unique seulement à son destinataire; quiconque l’a peut définir le mot de passe.

## Recovery codes
codes-title = Codes de récupération
codes-text = Conservez ces codes hors ligne. Chacun fonctionne une fois, à la place d’un code de l’authentificateur. Ils ne sont affichés que maintenant.
codes-continue = Je les ai conservés, continuer

## Organizations
orgs-title = Organisations
orgs-empty = Vous n’appartenez encore à aucune organisation.
orgs-create-title = Créer une organisation
orgs-create = Créer
field-org-name = Nom
field-slug = Nom court
field-slug-help = Lettres minuscules, chiffres et traits d’union, par exemple acme-corp.
col-name = Nom
col-slug = Nom court
col-status = État
col-your-role = Votre rôle
col-created = Créée le
org-status-active = Active
org-status-archived = Archivée
role-instance-owner = Propriétaire de l’instance

## One organization
org-title = Organisation
members-title = Membres
members-add-title = Ajouter un membre
members-add = Ajouter
org-invitation-issued = { $email } a été invité. Envoyez-lui ce lien :
col-administrator = Administrateur
col-role = Rôle
col-actions = Actions
member-role-for = Rôle de { $email }
member-change-role = Changer le rôle
member-remove = Retirer
member-remove-confirm = Retirer { $email } de cette organisation?
account-suspend = Suspendre
account-reactivate = Réactiver
account-reset-second-factor = Réinitialiser l’authentificateur
account-revoke-sessions = Fermer les sessions
account-suspend-confirm = Suspendre { $email }? Ses sessions prennent fin.
account-reactivate-confirm = Réactiver { $email }?
account-reset-second-factor-confirm = Réinitialiser l’authentificateur de { $email }? Ses sessions prennent fin et il doit le configurer de nouveau.
account-revoke-sessions-confirm = Fermer toutes les sessions de { $email }?
account-reset-issued = L’authentificateur de { $email } a été réinitialisé. Envoyez-lui ce lien :
org-manage-title = Gérer l’organisation
field-new-org-name = Nouveau nom
org-rename = Renommer
org-archive = Archiver l’organisation
org-archive-confirm = Archiver cette organisation?
role-owner = Propriétaire
role-administrator = Administrateur
role-approver = Approbateur
role-auditor = Auditeur
role-read-only = Lecture seule
admin-status-invited = Invité
admin-status-active = Actif
admin-status-suspended = Suspendu

## Profile
profile-title = Profil
profile-details = Renseignements
password-title = Mot de passe
password-text = Changer le mot de passe ferme vos autres sessions.
field-current-password = Mot de passe actuel
password-submit = Changer le mot de passe
password-changed = Le mot de passe a été changé.
codes-regenerate-title = Nouveaux codes de récupération
codes-regenerate-text = Les nouveaux codes remplacent les anciens, qui cessent de fonctionner.
codes-regenerate = Émettre de nouveaux codes

## Sessions
sessions-title = Sessions
col-kind = Type
col-last-seen = Dernière activité
col-expires = Expire le
col-source = Adresse d’origine
session-kind-console = Console
session-kind-cli = Ligne de commande
session-current = (cette session)
session-revoke = Fermer

## Tokens
tokens-title = Jetons d’API
tokens-text = Un jeton agit avec vos autorisations, pour les scripts et les intégrations. Il dure au plus 90 jours.
field-token-name = Nom du jeton
field-token-days = Valide pendant (jours)
tokens-create = Créer un jeton
tokens-secret-once = Copiez le jeton maintenant; il n’est affiché qu’une fois :
col-prefix = Commence par
token-revoke = Révoquer

## Audit
audit-title = Journal d’audit
audit-filter = Organisation
audit-all = Toutes les organisations
audit-verify = Vérifier la chaîne
audit-intact = La chaîne du journal est intacte ({ $count } entrées).
audit-broken = La chaîne du journal est rompue à l’entrée { $seq }.
audit-older = Entrées plus anciennes
col-when = Quand
col-actor = Auteur
col-action = Action
col-target = Cible
col-result = Résultat
result-success = Réussite
result-denied = Refusé
result-failed = Échec

## Instance
instance-title = Instance
instance-id = Identifiant de l’instance
instance-initialized = Initialisée le
instance-sessions = Limites des sessions
field-inactivity = Déconnecter après ce nombre de minutes sans activité (5 à 60)
field-absolute = Fermer les sessions ce nombre d’heures après la connexion (1 à 24)

## Errors
error-invalid = Certains renseignements ne sont pas valides.
error-not-found = Introuvable, ou non visible pour vous.
error-conflict = Cela entre en conflit avec l’état actuel.
error-not-initialized = Le Hub n’est pas initialisé.
error-already-initialized = Le Hub est déjà initialisé.
error-unauthenticated = La connexion a échoué, ou votre session a pris fin.
error-forbidden = Vous n’êtes pas autorisé à faire cela.
error-throttled = Trop de tentatives. Attendez un moment et réessayez.
error-internal = Le Hub a eu une erreur interne; consultez ses journaux.
error-bad-request = La demande n’a pas été comprise.
error-network = Le Hub est injoignable.
error-reauthentication-failed = Le mot de passe ou le code de l’authentificateur est incorrect.
error-password-mismatch = Les deux mots de passe diffèrent.
error-unexpected = Une erreur s’est produite.
error-with-detail = { $message } ({ $detail })
