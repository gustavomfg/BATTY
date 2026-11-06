# BATTY — direção: companheiro e ferramenta, ativado pelo terminal

Este documento registra a direção do projeto. Ele substitui a ideia de um pet
sempre presente no ambiente como objetivo final. Os poleiros reais do habitat
continuam válidos como etapa técnica, mas não definem mais o produto.

## Decisões

- **Ativação somente pelo terminal.** Nada de autostart, serviço, daemon, tray
  ou processo de fundo. O comando `batty` existe enquanto roda.
- **Papel duplo.** Ele é companheiro (presença afetuosa e discreta) e ferramenta:
  nota coisas que o usuário pode ter esquecido e, sob pedido, olha o monitor.
- **Estilo fofinho.** A arte deve ficar bonitinha, com proporções mais
  arredondadas. A identidade fica preservada: corpo roxo, olhos grandes, orelhas
  rosadas e a silhueta pendurada.
- **Captura** do monitor em foco, no momento do comando, pelo Noctalia.
- **Visão sem modelo na versão 1.** Regras sobre a captura, sem rede neural nem
  nada instalado. Um modelo local continua possível depois, se as regras não bastarem.
- **Nada é gravado.** A captura existe só em memória durante a análise. Nada é
  enviado para fora da máquina.
- **Nenhuma instalação sem autorização.** O sistema usa o que já está instalado.

## Versão 1 (implementada)

Comando: `batty` (binário do pacote `batpet-desktop`).

| Opção | Efeito |
| --- | --- |
| sem opções | fala no terminal, mostra o morcego por 8 s e sai |
| `--look` | também captura o monitor em foco e mede vermelho de erro |
| `--stay N` | tempo na tela em segundos; `0` mantém a janela até fechar |
| `--help` | uso |

Quando qualquer flag de desenvolvimento é usada (`--debug`, `--habitat-loop`,
`--flight-*`, `--review-*`), o comportamento antigo é mantido: a janela fica até
ser fechada.

### Sinais

- **Repositório:** `git status` da pasta onde `batty` foi chamado. Conta os
  arquivos alterados e a idade do mais antigo.
- **Hora local:** via `date +%H`.
- **Visão (só com `--look`):** a captura do monitor em foco é passada pelo pipe
  do Noctalia para `batty analyze`. Esse subcomando mede a fração de pixels de
  vermelho de erro. Ele grava só `red_fraction` em `$XDG_RUNTIME_DIR/batty/`,
  e o arquivo é apagado após a leitura.

### Decisão

Cada fala possível recebe uma pontuação. Só a maior é dita, se passar de 0,35:

| Fala | Gatilho | Pontuação |
| --- | --- | --- |
| Vermelho na tela | fração ≥ 0,3% | 0,9 |
| Alterações antigas | arquivos alterados há pelo menos 1 h | 0,4 a 0,8, crescendo até 1 dia |
| Hora tardia | 23h às 4h59 | 0,6 |

Sem nada acima do limiar, ele diz "nada urgente por agora." e fica quieto.
Os valores estão em `batpet-desktop/src/companion/brain.rs`.

### Reação

Qualquer fala faz o morcego usar o gesto de clique (encolher e espiar) logo ao
abrir. É uma reação provisória, enquanto não houver gestos próprios para avisos.

## Verificado nesta máquina

- `cargo test --workspace`: 101 testes passam (17 do protótipo de terminal e 84
  do desktop, incluindo 22 novos).
- Análise de PNG sintético: 400 pixels de erro em 120 mil dá `0,003333`, o valor
  esperado.
- `batty --stay 2` abre, fala e sai em cerca de 3 s.
- `batty --look`: captura, análise, fala e saída completas. Nenhuma imagem e
  nenhum arquivo temporário ficaram em disco.

## Limites atuais

- A análise não lê texto. Sabe que há vermelho forte, não o que diz.
- O limiar de vermelho ainda é uma aproximação. Ele foi escolhido para não
  confundir o roxo do morcego e o rosa do papel de parede. Precisa de calibração
  em telas reais de erro. O terminal mostra a fração medida em cada `--look`.
- O caminho passado ao Noctalia não pode ter espaços.
- A janela continua com `AlwaysOnTop` e posição superior direita. Para uso
  pontual, ainda precisa ser decidido se isso fica.
- Não há arte nova. A versão 1 usa a arte aprovada.

## Próximos passos

1. Calibrar o limiar de vermelho com capturas de erros reais do seu ambiente.
2. Decidir se a janela deve mesmo ficar sempre por cima e no canto.
3. Arte fofinha, preservando a identidade (fase 3).
4. Só depois, avaliar se um modelo local ajuda em algo que as regras não cobrem.

## Privacidade

A captura contém tudo que está visível, inclusive mensagens e senhas. Por isso a
captura é explícita, feita apenas quando `--look` é chamado. A imagem existe só
em memória durante a análise e nunca é gravada. Nada sai da máquina.
