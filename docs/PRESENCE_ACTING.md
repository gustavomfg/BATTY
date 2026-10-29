# Presença — primeira iteração de character acting

A base visual aprovada é o commit `8cd9437`. Os PNGs, recortes, cores, desenho
dos olhos e poses de repouso foram preservados. Não há Flight novo, diálogos,
menus ou serviços externos. A reação ao clique permanece a mesma.

## O que a comparação revelou

Ao aproximar o cursor, a versão anterior reduzia o deslocamento das pupilas
justamente quando a cabeça começava a seguir. Isso enfraquecia a leitura de
percepção. A saída zerava imediatamente o alvo dos olhos, ambas as orelhas
reagiam juntas, e os gestos de idle atrasados podiam disparar logo em seguida.
Um cursor imóvel mantinha a postura alerta indefinidamente.

## Decisões de atuação

- O olhar fica legível a distâncias menores. A suavização permanece contínua,
  mas a pose final tem um pixel lógico de amplitude e limiares distintos de
  entrada/saída para evitar tremores. Pupilas continuam dentro dos olhos.
- A cabeça espera 220 ms de direção estável antes de acompanhar. Movimentos
  curtos podem receber apenas um olhar. A recuperação também passa pela
  suavização, sem rotação ou deslocamento das garras.
- Uma orelha presta atenção primeiro. A outra só acompanha em proximidade
  maior. Não foram acrescentados desenhos ou eventos periódicos de orelha.
- Um cursor imóvel se torna familiar: após 1,2 s a postura começa a relaxar,
  durante aproximadamente 3 s. O olhar permanece observador. Uma única
  piscada lenta aos 2,8 s marca esse relaxamento; pequenos tremores do mouse
  não reiniciam o processo.
- O recuo tímido existente tem recuperação mais longa e uma breve meia
  pálpebra. Continua disparando na entrada da proximidade, com cooldown,
  sem repetir enquanto o cursor fica ali.
- Ao perder o cursor, ele mantém o alvo dos olhos por 650 ms; a postura
  conserva a direção por 550 ms e depois relaxa. Um breve piscar acompanha
  essa transição. A memória termina, sem procurar indefinidamente.
- Os timers de gestos espontâneos recebem uma margem após a interação. Isso
  evita uma sequência de movimentos atrasados assim que o mouse sai.
- Coordenadas inválidas ou fora da janela durante a inicialização contam
  como ausência. Esse caso apareceu no ensaio nativo em X11.

A skill `motion-design` orientou a separação entre perceber, acompanhar e
recuperar. O maior ganho veio da ordem e das pausas, com a amplitude anterior.
Respiração, asas, apoio, frequência básica de piscadas e clique permaneceram
deliberadamente discretos. Não foi acrescentada uma animação de asas para
preencher os períodos de quietude.

## Comparação e reprodução

[Antes/depois animado](visual-review/presence-before-after.gif) ·
[Momentos da sequência](visual-review/presence-poses.png)

O antes usa a base oficial, com apenas o mesmo código de captura de revisão.
O depois usa a iteração atual. Os quadros vêm do renderer Bevy. A cruz verde
adicionada à comparação indica o cursor efetivamente recebido, registrado
em `cursor.csv`; ela não faz parte do personagem ou da janela normal.

Ensaio controlado, portátil, com o mesmo trajeto de entrada:

```bash
cargo run -p batpet-desktop -- --review-dir /tmp/presence --review-presence
python tools/verify_review.py /tmp/presence
```

Ensaio Linux/X11 com movimentos reais do ponteiro, sem cliques:

```bash
cargo build -p batpet-desktop
python tools/review_native_cursor.py target/debug/batpet-desktop /tmp/presence-native
python tools/verify_review.py /tmp/presence-native
```

O helper requer X11/Xwayland, `xprop`, libX11 e libXtst. Ele abre uma pequena
janela temporária para receber o cursor fora do BATTY: Xwayland pode ignorar
warps para superfícies Wayland. Fecha essa janela e restaura o ponteiro no fim.
A janela de revisão usa fundo opaco porque o backend X11 deste ambiente não
aceitou transparência. O uso normal em Wayland continua transparente.

O trajeto dura 27 s: aproximação, acompanhamento, cinco segundos parado,
afastamento, sete segundos ausente, retorno breve e recuperação. O modo
normal não move o ponteiro nem grava capturas. O ensaio registra alvo pedido
e posição recebida; nos eventos nativos há atraso de amostragem entre ambos.

Os testes cobrem a precedência dos olhos sobre a cabeça, retorno após perda,
habituação sem piscadas repetidas, resistência a pequenos tremores e rejeição
de coordenadas fora da janela. A verificação de capturas confere blocos 8×8
uniformes e apoio fixo. O número de quads e texturas permanece igual; nenhum
asset é alocado por frame. Não foi feito benchmark comparativo de CPU/GPU.

Resultado desta passagem: 59 testes passaram; os 312 quadros da captura final
mantiveram a grade 8×8 e o apoio fixo. Os dois registros nativos confirmaram
movimentos dentro da janela e ausência durante o afastamento. A montagem
comparativa preserva essa diferença entre alvo pedido e cursor recebido.

## Limites

A percepção ainda se limita à janela de 320×320. Quando o mouse sai, só existe
uma lembrança breve da última posição, não acompanhamento global do desktop.
A grade de 32×32 limita nuances; acompanhar a cabeça verticalmente e articular
asas com maior independência exigiria poses adicionais, fora desta iteração.

Ainda é uma resposta a posição e movimento, com tempos escolhidos para essa
personalidade. Não infere intenção do usuário. Os intervalos podem ser
reconhecidos após observação prolongada, e a experiência de foco/posicionamento
continua dependente do compositor. Esta passagem deve ser avaliada como uma
primeira melhoria de presença, não como consciência real.
