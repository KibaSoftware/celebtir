"use client";
import { Children, cloneElement, isValidElement, Suspense, type ReactElement, type ReactNode } from "react";
import { useViewState } from "@/lib/useViewState";
import styles from "@/components/page/Page.module.css";

type Node = ReactElement<{children?: ReactNode; "data-sort-id"?: string; "aria-sort"?: "ascending" | "descending"}>;
/** Sort server-rendered cells by their original typed values, never formatted dollars. */
type Props = {children: ReactNode; values: Record<string,(string|number|null)[]>; label: string; defaultColumn?: number};
export function SortableTable(props: Props) {
  return <Suspense fallback={<div className={styles.scroll} role="region" aria-label={props.label} tabIndex={0}>{props.children}</div>}><SortedTable {...props} /></Suspense>;
}
function SortedTable({children, values, label, defaultColumn = 0}: Props) {
  const [view,update] = useViewState();
  const requested = /^column(\d+)$/.exec(view.sort.key);
  const column = requested ? Number(requested[1]) : defaultColumn;
  const desc = requested ? view.sort.dir === "desc" : defaultColumn !== 0;
  const transform = (nodes: ReactNode): ReactNode => Children.map(nodes,node => {
    if (!isValidElement(node)) return node;
    const element = node as Node;
    if(element.type === "thead") return cloneElement(element,{},Children.map(element.props.children,row=>{
      if(!isValidElement(row)) return row;
      const tr=row as Node;
      return cloneElement(tr,{},Children.map(tr.props.children,(cell,i)=>{
        if(!isValidElement(cell)) return cell;
        const th=cell as Node;
        return cloneElement(th,{"aria-sort":i===column ? desc ? "descending" : "ascending" : undefined},<button className={styles.sort} onClick={()=>update(v=>({...v,sort:{key:`column${i}`,dir:i===column && desc ? "asc" : "desc"}}))}>{th.props.children} <span aria-hidden="true">{i===column ? desc ? "↓" : "↑" : "↕"}</span></button>);
      }));
    }));
    if(element.type === "tbody") {
      const rows=Children.toArray(element.props.children) as Node[];
      rows.sort((a,b)=>{
        const av=values[a.props["data-sort-id"] ?? ""]?.[column] ?? null,bv=values[b.props["data-sort-id"] ?? ""]?.[column] ?? null;
        if(av===null) return bv===null ? 0 : 1;
        if(bv===null) return -1;
        const comparison=typeof av === "number" && typeof bv === "number" ? av-bv : String(av).localeCompare(String(bv));
        return desc ? -comparison : comparison;
      });
      return cloneElement(element,{},rows);
    }
    return cloneElement(element,{},transform(element.props.children));
  });
  return <div className={styles.scroll} role="region" aria-label={label} tabIndex={0}>{transform(children)}</div>;
}
