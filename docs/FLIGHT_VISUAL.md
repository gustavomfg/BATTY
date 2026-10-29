# Flight 0.2.1 — passagem visual

## Problema encontrado

O Flight Foundation movia o root corretamente, mas o rig continuava montando a
mesma silhueta vertical do `HangingIdle`. Mesmo com as garras ocultas, um frame
isolado ainda parecia um sprite pendurado sendo transportado.

Deslocar os recortes laterais existentes não resolveu a leitura: produzia duas
pontas afastadas, sem uma asa realmente aberta. A mudança de maior impacto foi
separar a apresentação aérea do rig de repouso.

## Separação visual

`FlightMotion` continua sendo a única fonte de posição e velocidade. O novo
`FlightVisualIntent` apenas deriva uma decisão de apresentação:

```text
FlightMotion → FlightVisualIntent → frame de asa / direção → rig
```

Ele calcula velocidade normalizada, velocidade vertical, direção horizontal e
um frame discreto. Não cria uma segunda trajetória nem altera o steering.

O conjunto mínimo tem quatro frames no sprite sheet `bat_flight_sheet.png`:

1. `Lift`: asas sobem, usado no início do takeoff e no upstroke;
2. `Spread`: asas abertas, a pose aérea estável;
3. `Power`: asas descem para o power stroke;
4. `Recover`: retorno mais curto antes do próximo ciclo.

As asas foram desenhadas na mesma grade de 1× lógico e na paleta roxa
existente. O corpo central é um recorte da textura aprovada, sem as garras do
hanging. Assim o rosto continua sendo o mesmo BATTY, enquanto a silhueta
ganha largura imediatamente quando ele se solta.

## Timing e direção

Durante `Flying`, o ciclo usa tempos diferentes para lift, spread, power e
recovery; não há troca uniforme entre dois sprites. Quando o movimento chega ao
alvo, a pose aberta fica estável durante o dwell. `Returning` mantém o ritmo
mais curto até a aproximação.

O sinal horizontal de `FlightMotion` controla `Sprite.flip_x` do asset aéreo.
Isso comunica a direção sem rotação livre, blur ou interpolação que quebre os
pixels. A velocidade vertical adiciona apenas o deslocamento de um pixel lógico
do corpo entre lift, power e recovery.

## Takeoff e Landing

No início do `Takeoff`, o rig pendurado permanece por `TAKEOFF_SUPPORT_HOLD`:
há uma pequena preparação ainda presa às garras. Logo depois, o frame `Lift`
troca para o asset aéreo e o suporte desaparece junto da quebra da silhueta.

No `Landing`, o asset aéreo segue ativo durante a aproximação. Ao entrar no
frame `Reach` e no raio final, ele é substituído pelo rig pendurado, as garras
voltam a aparecer e `finish_landing` alinha a âncora. A troca acontece perto do
perch, depois da desaceleração, para que o contato pareça uma aterrissagem.

## Bounds e validação visual

O asset aéreo ocupa 40×32 pixels lógicos; as bordas de asa foram mantidas um
pixel para dentro. Os alvos locais também foram reposicionados para que os
frames abertos caibam na janela 320×320 sem reduzir a silhueta. A captura final
foi verificada por bounds de alpha: nenhum frame aéreo ultrapassou a janela.

[Comparação antes/depois](visual-review/flight-before-after.png) ·
[quadros do ciclo observado](visual-review/flight-cycle.png)

O teste de revisão é:

```bash
cargo run -p batpet-desktop -- \
  --review-dir /tmp/batpet-flight-review --review-flight --flight-once
```

Os testes unitários cobrem seleção de frame, índices do sheet, direção,
valores finitos e as invariantes existentes de FlightMotion. `cargo fmt`,
`cargo check` e `cargo test` continuam sendo executados no workspace inteiro.

O sistema Alive não roda como idle completo durante o voo. O rosto, as pupilas,
a paleta e a personalidade reconhecível permanecem; respiração, gestos de
repouso e orelhas do hanging não competem com a batida das asas.

Ainda faltam poses desenhadas para banking, inclinação de corpo e uma asa
independente de cada lado. O asset atual comunica voo de frente, com quatro
frames e uma trajetória local; não há voo pelo desktop, obstáculos ou novos
perches nesta etapa.

A continuação focada em ritmo, apoio e contato está em
[FLIGHT_ACTING.md](FLIGHT_ACTING.md), com comparação animada da versão 320312a.
