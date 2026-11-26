export const selectableTableUi = {
  tbody: [
    "[&>tr]:data-[selectable=true]:focus-visible:bg-elevated/50",
    "not-forced-colors:[&>tr]:data-[selectable=true]:focus-visible:outline-none",
    "forced-colors:[&>tr]:data-[selectable=true]:focus-visible:-outline-offset-3"
  ],
  tr: "data-[selectable=true]:cursor-pointer"
};

export function onTableRowKeydown(event: KeyboardEvent) {
  const row = event.target;
  if (!(row instanceof HTMLTableRowElement) || row.tabIndex !== 0) return;

  event.preventDefault();
  if (!event.repeat) row.click();
}
