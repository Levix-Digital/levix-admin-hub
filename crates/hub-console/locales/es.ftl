## Shell
app-name = Levix Admin Hub
skip-to-content = Ir al contenido
loading = Cargando…
language = Idioma
dismiss = Cerrar
sign-out = Cerrar sesión
nav-label = Principal
nav-organizations = Organizaciones
nav-audit = Auditoría
nav-profile = Perfil
nav-sessions = Sesiones
nav-tokens = Tokens de API
nav-instance = Instancia
save = Guardar
saved = Guardado.

## Notices
notice-session-ended = Su sesión terminó. Inicie sesión de nuevo.
notice-signed-out = Ha cerrado la sesión.
notice-inactivity = Se cerró su sesión tras un periodo de inactividad.
notice-reenroll = Su autenticador se restableció. Configúrelo de nuevo para iniciar sesión.

## Inactivity
idle-title = ¿Sigue ahí?
idle-text = Su sesión se cerrará en un minuto por inactividad.
idle-stay = Mantener la sesión

## Not initialized and not found
not-initialized-title = Este Hub aún no está inicializado
not-initialized-text = Inicialícelo una vez en el servidor donde se ejecuta:
not-initialized-next = El comando muestra la huella del certificado y un enlace de configuración para el propietario de la instancia. Abra ese enlace aquí.
not-found-title = Página no encontrada
not-found-home = Ir a la página de inicio

## Sign-in
login-title = Iniciar sesión
login-submit = Iniciar sesión
field-email = Correo electrónico
field-password = Contraseña
field-code = Código del autenticador
field-code-help = El código de 6 dígitos de su aplicación de autenticación, o uno de sus códigos de recuperación.

## Links
link-title = Configure su cuenta
link-intro = Este enlace de { $purpose } es para { $email } y es válido hasta { $expires }.
purpose-setup = configuración
purpose-invitation = invitación
purpose-recovery = recuperación
enroll-intro = Primero, añada el Hub a su aplicación de autenticación.
enroll-start = Configurar el autenticador
enroll-title = Autenticador
enroll-scan = Escanee este código con su aplicación de autenticación.
enroll-qr-alt = Código QR del secreto del autenticador
enroll-manual = O introduzca este secreto a mano:
field-display-name = Nombre visible
field-new-password = Nueva contraseña
field-password-help = Al menos 12 caracteres. Se rechazan las contraseñas comunes.
field-confirm-password = Repita la nueva contraseña
link-submit = Guardar e iniciar sesión
link-deliver-help = Entregue este enlace de un solo uso solo a su destinatario; quien lo tenga puede definir la contraseña.

## Recovery codes
codes-title = Códigos de recuperación
codes-text = Guarde estos códigos fuera de línea. Cada uno funciona una vez, en lugar de un código del autenticador. Solo se muestran ahora.
codes-continue = Ya los guardé, continuar

## Organizations
orgs-title = Organizaciones
orgs-empty = Aún no pertenece a ninguna organización.
orgs-create-title = Crear una organización
orgs-create = Crear
field-org-name = Nombre
field-slug = Nombre corto
field-slug-help = Letras minúsculas, números y guiones, por ejemplo acme-corp.
col-name = Nombre
col-slug = Nombre corto
col-status = Estado
col-your-role = Su rol
col-created = Creada
org-status-active = Activa
org-status-archived = Archivada
role-instance-owner = Propietario de la instancia

## One organization
org-title = Organización
members-title = Miembros
members-add-title = Añadir un miembro
members-add = Añadir
org-invitation-issued = { $email } fue invitado. Envíele este enlace:
col-administrator = Administrador
col-role = Rol
col-actions = Acciones
member-role-for = Rol de { $email }
member-change-role = Cambiar rol
member-remove = Quitar
member-remove-confirm = ¿Quitar a { $email } de esta organización?
account-suspend = Suspender
account-reactivate = Reactivar
account-reset-second-factor = Restablecer autenticador
account-revoke-sessions = Cerrar sesiones
account-suspend-confirm = ¿Suspender a { $email }? Sus sesiones terminan.
account-reactivate-confirm = ¿Reactivar a { $email }?
account-reset-second-factor-confirm = ¿Restablecer el autenticador de { $email }? Sus sesiones terminan y debe configurarlo de nuevo.
account-revoke-sessions-confirm = ¿Cerrar todas las sesiones de { $email }?
account-reset-issued = El autenticador de { $email } se restableció. Envíele este enlace:
org-manage-title = Administrar la organización
field-new-org-name = Nuevo nombre
org-rename = Renombrar
org-archive = Archivar la organización
org-archive-confirm = ¿Archivar esta organización?
role-owner = Propietario
role-administrator = Administrador
role-approver = Aprobador
role-auditor = Auditor
role-read-only = Solo lectura
admin-status-invited = Invitado
admin-status-active = Activo
admin-status-suspended = Suspendido

## Profile
profile-title = Perfil
profile-details = Datos
password-title = Contraseña
password-text = Cambiar la contraseña cierra sus otras sesiones.
field-current-password = Contraseña actual
password-submit = Cambiar la contraseña
password-changed = La contraseña se cambió.
codes-regenerate-title = Nuevos códigos de recuperación
codes-regenerate-text = Los nuevos códigos reemplazan a los anteriores, que dejan de funcionar.
codes-regenerate = Generar nuevos códigos

## Sessions
sessions-title = Sesiones
col-kind = Tipo
col-last-seen = Última actividad
col-expires = Caduca
col-source = Dirección de origen
session-kind-console = Consola
session-kind-cli = Línea de comandos
session-current = (esta sesión)
session-revoke = Cerrar

## Tokens
tokens-title = Tokens de API
tokens-text = Un token actúa con sus permisos, para scripts e integraciones. Dura como máximo 90 días.
field-token-name = Nombre del token
field-token-days = Válido durante (días)
tokens-create = Crear un token
tokens-secret-once = Copie el token ahora; solo se muestra una vez:
col-prefix = Empieza por
token-revoke = Revocar

## Audit
audit-title = Auditoría
audit-filter = Organización
audit-all = Todas las organizaciones
audit-verify = Verificar la cadena
audit-intact = La cadena de auditoría está íntegra ({ $count } registros).
audit-broken = La cadena de auditoría está rota en el registro { $seq }.
audit-older = Registros anteriores
col-when = Cuándo
col-actor = Autor
col-action = Acción
col-target = Objetivo
col-result = Resultado
result-success = Éxito
result-denied = Denegado
result-failed = Fallido

## Instance
instance-title = Instancia
instance-id = Id de la instancia
instance-initialized = Inicializada
instance-sessions = Límites de las sesiones
field-inactivity = Cerrar sesión tras estos minutos sin actividad (5 a 60)
field-absolute = Terminar las sesiones estas horas después de iniciar sesión (1 a 24)

## Errors
error-invalid = Algún dato no es válido.
error-not-found = No encontrado, o no visible para usted.
error-conflict = Esto entra en conflicto con el estado actual.
error-not-initialized = El Hub no está inicializado.
error-already-initialized = El Hub ya está inicializado.
error-unauthenticated = El inicio de sesión falló, o su sesión terminó.
error-forbidden = No tiene permiso para hacer esto.
error-throttled = Demasiados intentos. Espere un momento e inténtelo de nuevo.
error-internal = El Hub tuvo un error interno; consulte sus registros.
error-bad-request = No se entendió la solicitud.
error-network = No se puede contactar con el Hub.
error-reauthentication-failed = La contraseña o el código del autenticador son incorrectos.
error-password-mismatch = Las dos contraseñas son distintas.
error-unexpected = Algo salió mal.
error-with-detail = { $message } ({ $detail })
