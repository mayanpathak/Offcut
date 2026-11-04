// Section 4: what Offcut does today (designimplementation §4, §6). Eight cards
// on a bento grid, each naming features that work on the live site. The
// small pictures in the cards are decoration, hidden from assistive
// technology: each card's text says the same.

import { messages } from "../../copy/messages";
import { LANDING_CLIPS } from "../../net/asset-fetch";
import { ArrowRight, Download } from "../icons";
import captionStylesPoster from "../assets/landing/captionStyles.webp";
import visualsPoster from "../assets/landing/visuals.webp";
import landing from "../styles/landing.module.css";
import styles from "../styles/landing-cards.module.css";
import { LoopVideo } from "./LoopVideo";
import { BentoCard, Chip, SectionHead } from "./parts";

/** The bars of the voice card's waveform: heights in a 0-1 range, noisy then clean. */
const WAVE_BARS = 28;
const WAVE_HEIGHT = 48;
const WAVE_GAP = 6;
const noisy = (i: number) => 0.25 + 0.75 * Math.abs(Math.sin(i * 1.7) * Math.cos(i * 0.6));
const clean = (i: number) => 0.3 + 0.55 * Math.abs(Math.sin(i * 0.45));
/** Seconds between the transcript card's words, as a recogniser would time them. */
const WORD_STEP_S = 0.4;
const ICON_SIZE = 16;

function Waveform() {
  const bars = Array.from({ length: WAVE_BARS }, (_, i) => i);
  const width = WAVE_BARS * WAVE_GAP;
  return (
    <svg className={styles.wave} viewBox={`0 0 ${String(width)} ${String(WAVE_HEIGHT)}`} aria-hidden="true" focusable="false">
      {[noisy, clean].map((shape, layer) => (
        <g key={layer} className={layer === 0 ? styles.waveNoisy : styles.waveClean}>
          {bars.map((i) => {
            const h = shape(i) * WAVE_HEIGHT;
            return <rect key={i} x={i * WAVE_GAP + 1} y={(WAVE_HEIGHT - h) / 2} width={3} height={h} rx={1.5} />;
          })}
        </g>
      ))}
    </svg>
  );
}

export function Features() {
  const copy = messages.landing;
  const cards = copy.features.cards;
  const mock = copy.features;
  return (
    <section className={landing.section} id="features" aria-labelledby="features-heading">
      <div className={landing.container}>
        <SectionHead
          eyebrow={copy.eyebrows.features}
          heading={copy.features.heading}
          headingId="features-heading"
          lead={copy.features.lead}
        />
        <div className={styles.bento}>
          <BentoCard index={0} className={styles.cardTall} title={cards.captions.title} body={cards.captions.body} chips={cards.captions.chips}>
            <div className={styles.tallClip}>
              <LoopVideo clip={LANDING_CLIPS.captionStyles} poster={captionStylesPoster} label={copy.clipLabels.captionStyles}>
                <div className={styles.captionStill} aria-hidden="true">
                  {cards.captions.chips.map((name, index) => (
                    <span key={name} className={styles.captionStillLine} data-style={index}>
                      {name}
                    </span>
                  ))}
                </div>
              </LoopVideo>
            </div>
          </BentoCard>

          <BentoCard index={1} className={styles.cardWide} title={cards.visuals.title} body={cards.visuals.body} chips={cards.visuals.chips}>
            <div className={styles.squareFrame}>
              <LoopVideo clip={LANDING_CLIPS.visuals} poster={visualsPoster} label={copy.clipLabels.visuals}>
                <div className={styles.visualsStill} aria-hidden="true">
                  <span className={styles.visualsNumber} />
                  <span className={styles.visualsBars} />
                </div>
              </LoopVideo>
            </div>
          </BentoCard>

          <BentoCard index={2} title={cards.voice.title} body={cards.voice.body}>
            <div className={styles.art}>
              <Waveform />
            </div>
          </BentoCard>

          <BentoCard index={3} title={cards.framing.title} body={cards.framing.body}>
            <div className={styles.art} aria-hidden="true">
              <div className={styles.cropFrame}>
                <span className={styles.cropWindow} />
              </div>
            </div>
          </BentoCard>

          <BentoCard index={4} className={styles.cardWide} title={cards.editor.title} body={cards.editor.body}>
            <div className={styles.art} aria-hidden="true">
              <p className={styles.editorLine}>
                <span>{mock.editorMock.line}</span>
                <span className={styles.editorSwap}>
                  <span className={styles.editorBefore}>{mock.editorMock.before}</span>
                  <span className={styles.editorAfter}>{mock.editorMock.after}</span>
                </span>
              </p>
            </div>
          </BentoCard>

          <BentoCard index={5} title={cards.transcription.title} body={cards.transcription.body}>
            <div className={styles.art} aria-hidden="true">
              <ol className={styles.transcript}>
                {mock.transcriptMock.map((word, index) => (
                  <li key={word}>
                    <span className={styles.transcriptTime}>{(index * WORD_STEP_S + WORD_STEP_S).toFixed(1)}</span>
                    {word}
                  </li>
                ))}
              </ol>
            </div>
          </BentoCard>

          <BentoCard index={6} title={cards.account.title} body={cards.account.body}>
            <div className={styles.art} aria-hidden="true">
              <span className={styles.mailChip}>
                {mock.mailChip}
                <ArrowRight size={ICON_SIZE} />
              </span>
            </div>
          </BentoCard>

          <BentoCard index={7} className={styles.cardWide} title={cards.export.title} body={cards.export.body}>
            <div className={styles.art} aria-hidden="true">
              <div className={styles.exportArt}>
                {cards.export.chips.slice(0, 2).map((badge) => (
                  <Chip key={badge}>{badge}</Chip>
                ))}
                <span className={styles.fileChip}>
                  <Download size={ICON_SIZE} />
                  {mock.fileChip}
                </span>
              </div>
            </div>
          </BentoCard>
        </div>
      </div>
    </section>
  );
}
