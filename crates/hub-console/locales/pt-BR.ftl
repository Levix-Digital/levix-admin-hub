## Shell
app-name = Levix Admin Hub
skip-to-content = Ir para o conteúdo
loading = Carregando…
language = Idioma
dismiss = Fechar
sign-out = Sair
nav-label = Principal
nav-organizations = Organizações
nav-audit = Auditoria
nav-profile = Perfil
nav-sessions = Sessões
nav-tokens = Tokens de API
nav-instance = Instância
save = Salvar
saved = Salvo.

## Notices
notice-session-ended = Sua sessão terminou. Entre novamente.
notice-signed-out = Você saiu.
notice-inactivity = Você saiu automaticamente após um período sem atividade.
notice-reenroll = Seu autenticador foi redefinido. Configure-o de novo para entrar.

## Inactivity
idle-title = Ainda está aí?
idle-text = Você vai sair em um minuto por falta de atividade.
idle-stay = Continuar conectado

## Not initialized and not found
not-initialized-title = Este Hub ainda não foi inicializado
not-initialized-text = Inicialize-o uma vez no servidor onde ele roda:
not-initialized-next = O comando mostra a impressão digital do certificado e um link de configuração para o proprietário da instância. Abra esse link aqui.
not-found-title = Página não encontrada
not-found-home = Ir para a página inicial

## Sign-in
login-title = Entrar
login-submit = Entrar
field-email = E-mail
field-password = Senha
field-code = Código do autenticador
field-code-help = O código de 6 dígitos do seu aplicativo autenticador, ou um dos seus códigos de recuperação.

## Links
link-title = Configure sua conta
link-intro = Este link de { $purpose } é para { $email } e vale até { $expires }.
purpose-setup = configuração
purpose-invitation = convite
purpose-recovery = recuperação
enroll-intro = Primeiro, adicione o Hub ao seu aplicativo autenticador.
enroll-start = Configurar o autenticador
enroll-title = Autenticador
enroll-scan = Leia este código com o seu aplicativo autenticador.
enroll-qr-alt = Código QR do segredo do autenticador
enroll-manual = Ou digite este segredo:
field-display-name = Nome de exibição
field-new-password = Nova senha
field-password-help = No mínimo 12 caracteres. Senhas comuns são recusadas.
field-confirm-password = Repita a nova senha
link-submit = Salvar e entrar
link-deliver-help = Entregue este link de uso único só ao seu destinatário; quem tiver o link pode definir a senha.

## Recovery codes
codes-title = Códigos de recuperação
codes-text = Guarde estes códigos fora do computador. Cada um funciona uma vez, no lugar de um código do autenticador. Eles só aparecem agora.
codes-continue = Já guardei, continuar

## Organizations
orgs-title = Organizações
orgs-empty = Você ainda não pertence a nenhuma organização.
orgs-create-title = Criar uma organização
orgs-create = Criar
field-org-name = Nome
field-slug = Nome curto
field-slug-help = Letras minúsculas, números e hífens, por exemplo acme-corp.
col-name = Nome
col-slug = Nome curto
col-status = Situação
col-your-role = Seu papel
col-created = Criada em
org-status-active = Ativa
org-status-archived = Arquivada
role-instance-owner = Proprietário da instância

## One organization
org-title = Organização
members-title = Membros
members-add-title = Adicionar um membro
members-add = Adicionar
org-invitation-issued = { $email } foi convidado. Envie este link:
col-administrator = Administrador
col-role = Papel
col-actions = Ações
member-role-for = Papel de { $email }
member-change-role = Mudar papel
member-remove = Remover
member-remove-confirm = Remover { $email } desta organização?
account-suspend = Suspender
account-reactivate = Reativar
account-reset-second-factor = Redefinir autenticador
account-revoke-sessions = Encerrar sessões
account-suspend-confirm = Suspender { $email }? As sessões dele terminam.
account-reactivate-confirm = Reativar { $email }?
account-reset-second-factor-confirm = Redefinir o autenticador de { $email }? As sessões terminam e ele precisa configurá-lo de novo.
account-revoke-sessions-confirm = Encerrar todas as sessões de { $email }?
account-reset-issued = O autenticador de { $email } foi redefinido. Envie este link:
org-manage-title = Gerenciar a organização
field-new-org-name = Novo nome
org-rename = Renomear
org-archive = Arquivar a organização
org-archive-confirm = Arquivar esta organização?
role-owner = Proprietário
role-administrator = Administrador
role-approver = Aprovador
role-auditor = Auditor
role-read-only = Somente leitura
admin-status-invited = Convidado
admin-status-active = Ativo
admin-status-suspended = Suspenso

## Profile
profile-title = Perfil
profile-details = Dados
password-title = Senha
password-text = Trocar a senha encerra suas outras sessões.
field-current-password = Senha atual
password-submit = Trocar a senha
password-changed = A senha foi trocada.
codes-regenerate-title = Novos códigos de recuperação
codes-regenerate-text = Os novos códigos substituem os antigos, que deixam de funcionar.
codes-regenerate = Gerar novos códigos

## Sessions
sessions-title = Sessões
col-kind = Tipo
col-last-seen = Última atividade
col-expires = Expira em
col-source = Endereço de origem
session-kind-console = Console
session-kind-cli = Linha de comando
session-current = (esta sessão)
session-revoke = Encerrar

## Tokens
tokens-title = Tokens de API
tokens-text = Um token age com as suas permissões, para scripts e integrações. Ele dura no máximo 90 dias.
field-token-name = Nome do token
field-token-days = Válido por (dias)
tokens-create = Criar um token
tokens-secret-once = Copie o token agora; ele só aparece uma vez:
col-prefix = Começa com
token-revoke = Revogar

## Audit
audit-title = Auditoria
audit-filter = Organização
audit-all = Todas as organizações
audit-verify = Verificar a cadeia
audit-intact = A cadeia da auditoria está íntegra ({ $count } registros).
audit-broken = A cadeia da auditoria está quebrada no registro { $seq }.
audit-older = Registros anteriores
col-when = Quando
col-actor = Autor
col-action = Ação
col-target = Alvo
col-result = Resultado
result-success = Sucesso
result-denied = Negado
result-failed = Falhou

## Instance
instance-title = Instância
instance-id = Id da instância
instance-initialized = Inicializada em
instance-sessions = Limites das sessões
field-inactivity = Sair após quantos minutos sem atividade (5 a 60)
field-absolute = Encerrar as sessões quantas horas após a entrada (1 a 24)

## Errors
error-invalid = Alguma informação não é válida.
error-not-found = Não encontrado, ou sem acesso para você.
error-conflict = Isso conflita com o estado atual.
error-not-initialized = O Hub não foi inicializado.
error-already-initialized = O Hub já foi inicializado.
error-unauthenticated = A entrada falhou, ou sua sessão terminou.
error-forbidden = Você não tem permissão para isso.
error-throttled = Tentativas demais. Aguarde um pouco e tente de novo.
error-internal = O Hub teve um erro interno; veja os logs dele.
error-bad-request = O pedido não foi entendido.
error-network = Não foi possível acessar o Hub.
error-password-mismatch = As duas senhas são diferentes.
error-unexpected = Algo deu errado.
error-with-detail = { $message } ({ $detail })
