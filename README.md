# BATTY

Um companheiro de desktop em pixel-art: um pequeno morcego roxo, pendurado,
curioso e um pouco tímido. Projeto experimental em Rust.

O workspace contém `batpet-desktop` (Bevy 0.19) e o protótipo independente
`batpet` para terminal (`crossterm`).

```bash
cargo run -p batpet-desktop
```

O mouse desperta atenção dentro da janela. O morcego olha primeiro, acompanha
com o rosto e reage à proximidade. Um clique faz um pequeno encolher e espiar,
com retorno ao repouso em 1,45 s. A tecla `F` inicia o ciclo experimental de
voo; `--flight-once` e `--flight-loop` permitem observá-lo repetidamente.

Em repouso há respiração, piscadas irregulares, olhares sustentados, reajustes
de postura e movimentos ocasionais das orelhas. Pausas e cooldowns evitam
sobrepor todos os gestos.

A identidade visual atual é a base oficial. A primeira iteração de presença
preserva esse desenho: os olhos percebem antes da cabeça, uma orelha acompanha
primeiro, e o morcego relaxa diante de um cursor parado. Ao sair da janela,
ele sustenta brevemente o último olhar antes de retomar o repouso.

[Comparação de character acting](docs/visual-review/presence-before-after.gif) ·
[Decisões da iteração de presença](docs/PRESENCE_ACTING.md)

## Direção visual

A textura original e sua paleta foram preservadas. Um rig leve recorta a mesma
textura em suporte, linhas do peito, rosto e pontas das orelhas. Olhos e
pálpebras são pequenos elementos geométricos na mesma grade. Não há atlas
externo pendente nem geração de imagens em runtime.

A janela mantém a escala 8×, nearest-neighbor e transparência. Os movimentos
visuais são quantizados na grade lógica; a respiração insere/remove uma linha,
sem rotação ou escala fracionária. O rig de repouso usa 27 quads e suas
texturas são montadas uma única vez; Flight acrescenta um sheet aéreo de quatro
frames, também carregado uma vez e sem alocações por frame.

[Comparação animada antes/depois](docs/visual-review/before-after.gif) ·
[Poses revisadas](docs/visual-review/poses.png) ·
[Decisões e limitações](docs/VISUAL_DIRECTION.md)

[Comparação Flight antes/depois](docs/visual-review/flight-before-after.png) ·
[quadros do primeiro ciclo](docs/visual-review/flight-cycle.png)

## Flight 0.2

O voo agora tem uma fundação de movimento separada da aparência. O estado passa
por `HangingIdle → Takeoff → Flying → Return → Landing → HangingIdle`. Cada
morcego possui um `FlightMotion` com posição, velocidade, aceleração, limites e
alvo de chegada; o steering usa aceleração limitada e desaceleração progressiva,
em vez de uma senoide ou de uma trajetória por frame. O local de repouso é um
`Perch`, que guarda a âncora das garras e um ponto de aproximação para o pouso.

Durante a saída, o suporte permanece por uma breve antecipação e então se oculta.
Na aproximação final ele reaparece antes da chegada exata, para que a relação
entre garras e perch seja restabelecida. A posição renderizada continua
quantizada em 8×; a simulação mantém valores contínuos e usa delta time.

A passagem visual 0.2.1 acrescenta quatro frames de asas abertas derivados de
`FlightMotion`. O corpo e o rosto continuam usando a identidade aprovada, mas
um frame aéreo substitui o rig pendurado assim que as garras soltam. A direção
usa flip horizontal e o ritmo separa lift, spread, power e recovery. Veja as
[decisões da passagem visual](docs/FLIGHT_VISUAL.md).

O 0.2 usava um perch e alvos locais; essa fundação agora é executada sobre o
Habitat 0.3. O habitat de desenvolvimento contém dois apoios artificiais,
mantém `current` e `destination` por ID e alterna A → B → A. A escolha produz um
`FlightPlan` de posições; Flight executa a mesma sequência e o pouso atualiza a
referência usada por Alive.

Para observar viagens repetidas com logs:

```bash
cargo run --bin batpet-desktop -- --debug --habitat-loop
```

`--flight-loop` continua aceito como alias; `--flight-once` faz uma viagem. Com
a janela focada, `F` escolhe o próximo perch e `1`/`2` direcionam para A/B. A
fundação e os limites da janela atual estão descritos em
[docs/HABITAT_FOUNDATION.md](docs/HABITAT_FOUNDATION.md); o movimento original
continua documentado em [docs/FLIGHT_FOUNDATION.md](docs/FLIGHT_FOUNDATION.md).

## Validação e revisão visual

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p batpet-desktop -- --debug
```

Para repetir a sequência visual no renderer real:

```bash
cargo run -p batpet-desktop -- --review-dir /tmp/batpet-review
cargo run -p batpet-desktop -- --review-dir /tmp/batpet-review-light --review-light
```

Para verificar a grade e o apoio nas capturas (requer Pillow):

```bash
python tools/verify_review.py /tmp/batpet-review
```

Para capturar um ciclo de voo no renderer real, use o modo opt-in junto do
disparador de ciclo único:

```bash
cargo run -p batpet-desktop -- --review-dir /tmp/batpet-flight-review --review-flight --flight-once
```

O modo de revisão simula atenção à esquerda/direita, proximidade e reação ao
clique, salva capturas a 12 fps e encerra após 13 segundos. Também verifica o
retorno ao estado de repouso. A captura não roda no uso normal. A sequência usa
os timers existentes; pequenas diferenças de quadros podem ocorrer conforme
o tempo de inicialização e o frame rate.

Encerre o aplicativo com `Ctrl+C` ou fechando a janela. Always-on-top e a
posição superior direita são solicitações ao compositor; KDE/Wayland pode
ignorá-las. O cursor continua limitado à área da janela.
