> **Especificação histórica, substituída pelo rig de pixels.** O renderer atual
> usa recortes de `bat_idle.png` e pálpebras geométricas, sem carregar os atlases
> descritos abaixo. Consulte [a direção visual](../../../docs/VISUAL_DIRECTION.md).

# BATTY — primeira folha de poses `HangingIdle`

Este arquivo descreve a arte necessária para substituir o fallback atual
`bat_idle.png`. A folha não deve ser adicionada ao jogo até que os frames sejam
revisados visualmente em escala 1× e 8×.

Os arquivos `bat_hanging_body.png` e `bat_hanging_face.png` ainda não fazem
parte do repositório nesta etapa; esta é uma especificação de produção, não
uma arte aprovada.

## Folhas e dimensões

### Corpo

- arquivo: `bat_hanging_body.png`
- formato: PNG indexed/RGBA com transparência binária
- dimensões: `320×32` pixels (`10×1` células de `32×32`)
- ordem das células:

  1. `neutral`
  2. `breathe_in`
  3. `breathe_out`
  4. `wing_adjust`
  5. `ear_twitch_left`
  6. `ear_twitch_right`
  7. `attention_left`
  8. `attention_right`
  9. `attentive`
  10. `very_near`

### Face

- arquivo: `bat_hanging_face.png`
- formato: PNG indexed/RGBA com transparência binária
- dimensões: `64×32` pixels (`2×1` células de `32×32`)
- ordem das células:

  1. `blink_half`
  2. `blink_closed`

A face é uma camada filha alinhada ao mesmo `Anchor::TOP_CENTER` do corpo.
Fora das pálpebras, cada célula deve ser transparente. A pupila continua em
uma camada separada para preservar o tracking contínuo.

O acting calcula `head_offset` continuamente, mas o personagem atual ainda é
um sprite monolítico. Nesta folha, a atuação da cabeça deve ser desenhada nas
poses `attention_left/right` sem mover os sockets dos olhos; uma camada de
cabeça separada só deve ser criada em uma etapa posterior se a arte exigir
paralaxe real.

## Invariantes de todos os frames

- vista frontal, morcego pendurado de cabeça para baixo; não produzir vistas
  laterais ou traseiras nesta fase;
- silhueta, anatomia, orientação do rosto, asas, orelhas e pés devem ser
  coerentes entre si;
- as garras/pés que seguram o topo são o ponto de apoio: os pixels de suporte
  devem permanecer na mesma posição em todas as células;
- mesma caixa `32×32`, mesmo `Anchor::TOP_CENTER`, sem crop variável e sem
  padding diferente entre frames;
- sockets/olhos abertos devem permanecer na mesma coordenada, salvo a arte da
  pálpebra nas duas células da folha facial;
- fundo realmente transparente, sem halo, sem anti-aliasing e sem pixels
  semitransparentes;
- palette curta e controlada, preservando roxos escuros, roxos médios,
  highlights rosados e o contraste dos olhos; evitar dezenas de cores quase
  iguais;
- nenhuma rotação, espelhamento automático ou deformação de perspectiva.

## O que muda por frame

- `neutral`: home pose fofa; cabeça grande, corpo compacto, asas relaxadas,
  orelhas legíveis e pés claramente presos ao topo;
- `breathe_in`: peito/barriga expande no máximo 1 pixel lógico; cabeça e asas
  acompanham com uma alteração menor; garras ficam fixas;
- `breathe_out`: retorno/compressão correspondente, sem deslocar o ponto de
  apoio;
- `wing_adjust`: pequeno ajuste de uma asa e do torso, alterando a silhueta
  sem mover o morcego inteiro;
- `ear_twitch_left/right`: apenas a orelha indicada faz o twitch, com uma
  mudança pequena e legível; a outra não deve ser um espelho automático;
- `attention_left/right`: olhos continuam na camada contínua, enquanto cabeça,
  orelhas e torso dão uma resposta lateral mínima; a vista ainda é frontal e
  as garras não saem do apoio;
- `attentive`: orelhas mais alertas e postura curiosa, sem alterar a orientação
  frontal nem deixar o personagem permanentemente tenso;
- `very_near`: compressão/recuo curto e fofo para a reação “opa”; manter o
  suporte das garras fixo e evitar uma pose de susto exagerado;
- `blink_half`: pálpebras cobrindo parcialmente os olhos;
- `blink_closed`: pálpebras fechadas, ainda com expressão fofa e sem apagar o
  rosto inteiro.

## Critério de aprovação

Verificar cada célula em `1×`, `8×` nearest-neighbor e sobre fundos claro e
escuro. Rejeitar a folha se o suporte das garras parecer flutuar, se o rosto
parecer mudar de orientação, se as asas perderem a leitura ou se o downscale
introduzir pixels suaves. A folha deve continuar sendo uma sequência frontal
coerente, não dez ilustrações independentes.
