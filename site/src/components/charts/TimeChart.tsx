"use client";

import { scaleLinear, scalePoint } from "d3-scale";
import { area, line, stack } from "d3-shape";
import { useId, useState, type KeyboardEvent, type PointerEvent } from "react";
import { calendarDays, type CalendarDay } from "@/lib/calendar";
import { formatAmount, formatUsd } from "@/lib/format";
import { surfaces } from "@/lib/tokens";
import styles from "./TimeChart.module.css";

export interface Series {
  key: string;
  label: string;
  color: string;
  /** Overlay lines: an SVG dash pattern, so lines differ without colour. */
  dash?: string;
}

export interface Point {
  date: string;
  values: Record<string, number>;
}

const W = 640;
const H = 220;
const M = { top: 12, right: 16, bottom: 26, left: 64 };
const SURFACE = surfaces.surface;

export type Unit = "usd" | "count" | "amount";
const FORMATS: Record<Unit, (n: number) => string> = { usd: formatUsd, count: (n) => String(Math.round(n)), amount: formatAmount };
const LABEL_MIN_PX = 14;

/** A legend key: a filled square, or for overlay lines a short dashed stroke. */
function Swatch({ s, line: asLine }: { s: Series; line: boolean }) {
  if (!asLine) return <span className={styles.swatch} style={{ background: s.color }} aria-hidden="true" />;
  return (
    <svg width="22" height="10" aria-hidden="true" className={styles.lineSwatch}>
      <line x1="1" x2="21" y1="5" y2="5" stroke={s.color} strokeWidth="2" strokeDasharray={s.dash || undefined} />
    </svg>
  );
}

/**
 * A line (one series), stacked areas (several), or overlay lines (`overlay`,
 * several series on one scale, gaps where a series has no value) over time, with one y-axis,
 * a crosshair tooltip that follows the pointer or the arrow keys, a legend for
 * two or more series, direct labels for up to four, and a table view (ADR 016).
 */
export function TimeChart({
  title,
  description,
  series,
  points,
  unit,
  overlay = false,
}: {
  title: string;
  description: string;
  series: Series[];
  points: Point[];
  /** A name, not a function, so server components can render the chart. */
  unit: Unit;
  /** Lines on one scale instead of stacked areas (UI-18). */
  overlay?: boolean;
}) {
  const format = FORMATS[unit];
  // Overlay series can lack a day; stacked and single series count it as zero.
  const show = (v: number | undefined) => (v === undefined ? (overlay ? "–" : format(0)) : format(v));
  const id = useId();
  const [active, setActive] = useState<number | null>(null);
  const stacked = series.length > 1 && !overlay;
  const direct = series.length > 1 && series.length <= 4;
  const right = direct ? (overlay ? 120 : 96) : M.right;
  // Calendar-true axis: a day without a snapshot keeps its slot and is drawn
  // as a gap, never joined across as if it had been measured.
  const observed = [...points].sort((a, b) => a.date.localeCompare(b.date));
  const days: CalendarDay<Point>[] = calendarDays(observed);
  const dates = days.map((p) => p.date);

  const x = scalePoint<string>().domain(dates).range([M.left, W - right]);
  const layers = stack<CalendarDay<Point>, string>()
    .keys(series.map((s) => s.key))
    .value((p, k) => p.values[k] ?? 0)(days);
  const top = Math.max(
    1,
    ...observed.map((p) =>
      stacked ? series.reduce((sum, s) => sum + (p.values[s.key] ?? 0), 0) : Math.max(0, ...series.map((s) => p.values[s.key] ?? 0)),
    ),
  );
  // Overlay name labels at each line's last value, nudged apart.
  const endLabels = overlay
    ? series
        .map((s) => {
          const last = [...observed].reverse().find((p) => p.values[s.key] !== undefined);
          return last ? { s, y: 0, value: last.values[s.key]! } : null;
        })
        .filter((l) => l !== null)
    : [];
  const y = scaleLinear().domain([0, top]).nice(4).range([H - M.bottom, M.top]);
  endLabels.forEach((l) => (l.y = y(l.value)));
  endLabels.sort((a, b) => a.y - b.y);
  endLabels.forEach((l, i) => {
    if (i > 0 && l.y - endLabels[i - 1]!.y < 13) l.y = endLabels[i - 1]!.y + 13;
  });
  const ticks = y.ticks(4).filter((t) => unit !== "count" || Number.isInteger(t));
  // F9: ticks sit on dates that have an observation, never on a gap day.
  const seen = observed.map((p) => p.date);
  const candidates = seen.length <= 4 ? seen : [seen[0]!, seen[Math.floor(seen.length / 2)]!, seen[seen.length - 1]!];
  const xLabels = candidates.filter((d, i) => {
    if (i === candidates.length - 1 || i === 0) return true;
    const next = candidates[i + 1]!;
    const prev = candidates[i - 1]!;
    return Math.abs((x(next) ?? 0) - (x(d) ?? 0)) >= 90 && Math.abs((x(d) ?? 0) - (x(prev) ?? 0)) >= 90;
  });

  const pick = (e: PointerEvent<SVGSVGElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * W;
    let best = 0;
    dates.forEach((d, i) => {
      if (Math.abs((x(d) ?? 0) - px) < Math.abs((x(dates[best]!) ?? 0) - px)) best = i;
    });
    setActive(best);
  };
  const keys = (e: KeyboardEvent<SVGSVGElement>) => {
    const last = dates.length - 1;
    if (e.key === "ArrowLeft") setActive((a) => Math.max(0, (a ?? last) - 1));
    else if (e.key === "ArrowRight") setActive((a) => Math.min(last, (a ?? -1) + 1));
    else if (e.key === "Escape") setActive(null);
    else return;
    e.preventDefault();
  };
  const at = active === null ? null : days[active];
  const ax = at ? (x(at.date) ?? 0) : 0;

  return (
    <figure className={styles.figure} aria-labelledby={`${id}-t`}>
      <figcaption>
        <span id={`${id}-t`} className={styles.title}>
          {title}
        </span>
        <span className={styles.description}>{description}</span>
      </figcaption>
      {series.length > 1 ? (
        <ul className={styles.legend} aria-label="Legend">
          {(overlay ? series : [...series].reverse()).map((s) => (
            <li key={s.key}>
              <Swatch s={s} line={overlay} />
              {s.label}
              {overlay && points.every((p) => p.values[s.key] === undefined) ? " (no data)" : ""}
            </li>
          ))}
        </ul>
      ) : null}
      <div className={styles.plot}>
        <svg
          viewBox={`0 0 ${W} ${H}`}
          className={styles.svg}
          role="img"
          aria-label={`${title}. Use the left and right arrow keys to read each day.`}
          tabIndex={0}
          onPointerMove={pick}
          onPointerLeave={() => setActive(null)}
          onKeyDown={keys}
          onBlur={() => setActive(null)}
        >
          {ticks.map((t) => (
            <g key={t}>
              <line x1={M.left} x2={W - right} y1={y(t)} y2={y(t)} className={t === 0 ? styles.baseline : styles.grid} />
              <text x={M.left - 8} y={y(t)} className={styles.tick} textAnchor="end" dominantBaseline="middle">
                {format(t)}
              </text>
            </g>
          ))}
          {xLabels.map((d, i) => (
            <text
              key={d}
              x={x(d)}
              y={H - 6}
              className={styles.tick}
              textAnchor={xLabels.length > 1 && i === 0 ? "start" : i === xLabels.length - 1 && xLabels.length > 1 ? "end" : "middle"}
            >
              {d}
            </text>
          ))}
          {stacked
            ? layers.map((layer, i) => {
                const s = series[i]!;
                const shape = area<(typeof layer)[number]>()
                  .defined((d) => !d.data.missing)
                  .x((d) => x(d.data.date) ?? 0)
                  .y0((d) => y(d[0]))
                  .y1((d) => y(d[1]));
                return <path key={s.key} d={shape(layer) ?? ""} fill={s.color} stroke={SURFACE} strokeWidth={2} />;
              })
            : series.map((s) => {
                const shape = line<CalendarDay<Point>>()
                  .defined((d) => !d.missing && (!overlay || d.values[s.key] !== undefined))
                  .x((d) => x(d.date) ?? 0)
                  .y((d) => y(d.values[s.key] ?? 0));
                return (
                  <path
                    key={s.key}
                    d={shape(days) ?? ""}
                    fill="none"
                    stroke={s.color}
                    strokeWidth={2}
                    strokeDasharray={s.dash || undefined}
                    strokeLinejoin="round"
                  />
                );
              })}
          {overlay
            ? endLabels.map((l) => (
                <text key={l.s.key} x={W - right + 8} y={l.y} className={styles.direct} dominantBaseline="middle">
                  {l.s.label}
                </text>
              ))
            : null}
          {direct && stacked
            ? layers.map((layer, i) => {
                const last = layer[layer.length - 1];
                if (!last || y(last[0]) - y(last[1]) < LABEL_MIN_PX) return null;
                return (
                  <text key={series[i]!.key} x={W - right + 8} y={(y(last[0]) + y(last[1])) / 2} className={styles.direct} dominantBaseline="middle">
                    {series[i]!.label}
                  </text>
                );
              })
            : null}
          {at ? (
            <g aria-hidden="true">
              <line x1={ax} x2={ax} y1={M.top} y2={H - M.bottom} className={styles.crosshair} />
              {!stacked && !at.missing
                ? series
                    .filter((s) => at.values[s.key] !== undefined || !overlay)
                    .map((s) => (
                    <circle key={s.key} cx={ax} cy={y(at.values[s.key] ?? 0)} r={4} fill={s.color} stroke={SURFACE} strokeWidth={2} />
                  ))
                : null}
            </g>
          ) : null}
        </svg>
        {at ? (
          <div
            className={styles.tooltip}
            role="status"
            style={{ left: `${(ax / W) * 100}%`, transform: ax > W / 2 ? "translateX(calc(-100% - 12px))" : "translateX(12px)" }}
          >
            <div className={styles.tipDate}>{at.date}</div>
            {at.missing ? <div className={styles.tipMissing}>No snapshot for this day</div> : null}
            {(at.missing ? [] : overlay ? series : [...series].reverse()).map((s) => (
              <div key={s.key} className={styles.tipRow}>
                <Swatch s={s} line={overlay} />
                <span>{s.label}</span>
                <span className={styles.tipValue}>{show(at.values[s.key])}</span>
              </div>
            ))}
          </div>
        ) : null}
      </div>
      <details className={styles.table}>
        <summary>Show as table</summary>
        <table>
          <caption className={styles.srOnly}>{title}</caption>
          <thead>
            <tr>
              <th scope="col">Date</th>
              {series.map((s) => (
                <th key={s.key} scope="col">
                  {s.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {observed.map((p) => (
              <tr key={p.date}>
                <th scope="row">{p.date}</th>
                {series.map((s) => (
                  <td key={s.key}>{show(p.values[s.key])}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </figure>
  );
}
