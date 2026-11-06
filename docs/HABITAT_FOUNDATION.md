# Habitat 0.3 — fundação

## Dependências encontradas antes da generalização

O 0.2 guardava um único `Perch` como componente do morcego. `start_takeoff` e
`start_flying` usavam a âncora desse componente; `start_returning` seguia seu
`approach_target`; `start_landing` e `finish_landing` voltavam à mesma âncora.
Não havia identidade, catálogo ou escolha de destino.

O retorno ao spawn também estava escondido em Alive: `IdleMotion.base_translation`
era inicializado uma vez em `rendering::setup`, e `apply_idle_motion` reescrevia
essa posição a cada frame pendurado. O rig usava o mesmo `Perch.anchor` para
decidir quando mostrar o suporte durante Landing. Apenas atualizar
`FlightMotion.position` no pouso deixaria esses dois caminhos apontando para A.

Flight simula em coordenadas da câmera local, centralizada na janela: `(0, 0)`
fica no centro, X cresce para a direita e Y para cima. `Transform` usa esse
mesmo espaço; somente a posição renderizada é quantizada na grade de 8 px. A
janela Bevy tem 320×320, escala sobrescrita para 1,0 e é posicionada no canto
superior direito usando dimensões físicas do monitor. Essa posição nativa da
janela não participa da simulação.

O renderer não oferece click-through. Transparência, foco, always-on-top e
posição dependem do suporte do compositor; no Wayland a posição absoluta pode
ser ignorada. A área pequena mantém input e custo do compositor limitados, mas
também recorta qualquer voo que ultrapasse 320×320. O habitat de desenvolvimento
precisa caber nesse quadro até existir uma fronteira explícita para coordenadas
do desktop.

## Modelo de Habitat

`Habitat` é um recurso ECS com um catálogo pequeno de `Perch`, o ID do perch
atual, o ID de destino durante a viagem e a contagem de pousos da revisão. Cada
`Perch` tem ID estável, âncora, ponto absoluto de aproximação, tipo de apoio,
orientação e disponibilidade. Os dois locais de desenvolvimento são apoios
suspensos artificiais A e B, a 48 px um do outro dentro da janela atual.

A âncora diz onde o morcego termina pendurado; `approach` diz onde desacelerar e
alinhá-lo antes do contato. `surface` e `facing` pertencem ao perch. Os dois
usam o mesmo tipo de apoio; B olha para o lado oposto para provar que a
orientação vem do destino atual. Uma disponibilidade booleana basta para a
seleção de desenvolvimento.

`Habitat` guarda `current` e `destination` como `PerchId`, sem acoplar a entidade
Bat a um componente de spawn. `F` escolhe o próximo destino disponível em ordem
de catálogo, sempre diferente do atual. `1` e `2` selecionam A e B durante o
desenvolvimento. `--habitat-loop` repete A → B → A; `--flight-loop` continua
aceito como alias. `--flight-once` e `F` também usam a seleção determinística.

Ao escolher, Habitat monta um `FlightPlan` composto somente por posições:
origem, waypoint de Takeoff, waypoint de voo, aproximação e pouso. O plano é a
fronteira entre escolha e execução; `flight.rs` recebe os pontos e não consulta
IDs, disponibilidade ou origem do catálogo. `FlightMotion` continua sendo a
única simulação, com o mesmo arrive steering, aceleração limitada, pausa aérea e
renderização em grade de 8 px. A pequena tolerância subpixel de chegada evita
que alvos muito próximos parem pouco antes do raio por causa do campo arrive.
O estado existente `Returning` (rótulo de log `Return`) executa a aproximação
do destino.

Quando Landing termina, `complete_habitat_landing` confirma o destino como
`current`. `finish_landing` ancora `FlightMotion`, `Transform` e
`IdleMotion.base_translation` no ponto do plano. O rig usa esse destino para
mostrar as garras durante a aproximação; Alive volta a rodar pelos estados
existentes. Respiração, blink, atenção ao cursor e character acting seguem
dependendo do estado e da posição transform atual. `apply_idle_motion` lê a
orientação do perch atual para manter o morcego voltado ao lado apropriado.

## Coordenadas atuais

Âncoras e pontos do `FlightPlan` são posições da simulação local à câmera, em
pixels lógicos: origem no centro da janela, X para a direita e Y para cima. O
`Transform` do root usa esse mesmo espaço. `FlightMotion.position` permanece
contínua; `render_position` apenas quantiza a cópia enviada ao `Transform`.
`window_cursor_to_simulation` converte o input lógico Bevy, que começa no canto
superior esquerdo e cresce para baixo, para esse espaço local.

`WindowPlacement` é outro espaço: a janela é posicionada com coordenadas e
dimensões físicas do monitor. A escala da resolução é sobrescrita para 1,0,
mas a simulação ainda não soma a origem física da janela nem converte pontos do
desktop. Isso é intencional: os perches artificiais provam a arquitetura sem
fazer o voo conhecer o compositor.

## Janela e próxima fronteira

A janela atual é 320×320, transparente, sem decoração, não redimensionável e
solicita always-on-top. A posição superior direita usa o primeiro monitor
exposto por Bevy. Transparência, foco e always-on-top dependem do compositor; o
posicionamento absoluto pode ser ignorado no Wayland. O renderer não configura
click-through, então a superfície ainda participa do input dentro de seus
limites, e `F`/`1`/`2` exigem foco da janela. O quadro pequeno limita o custo de
composição, mas recorta o morcego se um destino ou uma asa ultrapassar suas
bordas. Um overlay de tela inteira mudaria a área de input e aumentaria a
superfície transparente composta pelo compositor.

Para substituir os pontos artificiais por lugares do desktop, a próxima etapa
precisa fornecer perches em coordenadas físicas do desktop e uma transformação
explícita que considere origem da janela, monitor e escala. Também precisará
decidir como a cena acompanha esses pontos sem ampliar silenciosamente a
superfície transparente: mover/redimensionar a janela ou adotar uma estratégia
de overlay requer rever clipping, foco, click-through e custo do compositor.
Essas decisões ficam fora deste milestone.

## Como observar e validar

Execute o loop no renderer normal:

```bash
cargo run --bin batty -- --debug --habitat-loop
```

Os logs registram origem, destino, entrada nos estados e perch atual após o
pouso. Para capturar três viagens completas e encerrar automaticamente:

```bash
cargo run --bin batty -- --debug \
  --review-dir /tmp/batpet-habitat-review --review-habitat --habitat-loop
```

Os testes verificam o catálogo, seleção determinística e manual, destino de
aproximação, commit no pouso, atualização do anchor usado por Alive, convergência
de alvos próximos, ciclo repetido sem deriva e valores finitos. As validações do
workspace são `cargo fmt --all`, `cargo check --workspace`,
`cargo test --workspace` e `git diff --check`.

Na revisão do renderer desta fase, três viagens terminaram com os logs
A → B → A → B; foram capturados 144 quadros. B aparece 48 px abaixo de A, o
repouso continua visível em ambos e os quadros de voo mantêm o asset existente.
