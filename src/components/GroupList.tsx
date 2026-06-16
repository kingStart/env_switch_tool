import { useI18n, t } from "../i18n";

interface GroupInfo {
  name: string;
  description: string;
  active: boolean;
  priority: number;
  variable_count: number;
}

interface Props {
  groups: GroupInfo[];
  selectedGroup: string | null;
  onSelect: (name: string) => void;
  onToggle: (name: string, active: boolean) => void;
  onDelete: (name: string) => void;
}

export function GroupList({ groups, selectedGroup, onSelect, onToggle, onDelete }: Props) {
  const { messages } = useI18n();

  if (groups.length === 0) {
    return (
      <div className="px-5 py-10 text-center">
        <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-slate-100 dark:bg-gray-700 flex items-center justify-center">
          <svg className="w-6 h-6 text-slate-300 dark:text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </div>
        <p className="text-sm text-slate-400 dark:text-gray-500">{messages.no_groups}</p>
        <p className="text-xs text-slate-300 dark:text-gray-600 mt-1">{messages.no_groups_hint}</p>
      </div>
    );
  }

  return (
    <div className="py-2">
      {groups.map((group) => {
        const isSelected = selectedGroup === group.name;
        return (
          <div
            key={group.name}
            onClick={() => onSelect(group.name)}
            className={`mx-2 mb-1 px-3 py-2.5 rounded-lg cursor-pointer transition-all duration-150 group ${
              isSelected
                ? "bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-700"
                : "hover:bg-slate-50 dark:hover:bg-gray-700/50 border border-transparent"
            }`}
          >
            <div className="flex items-center justify-between">
              <span className={`font-medium text-sm truncate ${
                isSelected ? "text-indigo-700 dark:text-indigo-300" : "text-slate-700 dark:text-gray-200"
              }`}>
                {group.name}
              </span>
              <div className="flex items-center gap-1.5">
                <button
                  onClick={(e) => { e.stopPropagation(); onToggle(group.name, group.active); }}
                  className={`relative w-8 h-[18px] rounded-full transition-colors duration-200 ${
                    group.active ? "bg-emerald-400" : "bg-slate-200 dark:bg-gray-600"
                  }`}
                  title={group.active ? messages.disable : messages.enable}
                >
                  <span className={`absolute top-[2px] left-[2px] w-[14px] h-[14px] bg-white rounded-full shadow-sm transition-transform duration-200 ${
                    group.active ? "translate-x-[14px]" : ""
                  }`} />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    if (confirm(t(messages.delete_confirm, { name: group.name }))) {
                      onDelete(group.name);
                    }
                  }}
                  className="w-5 h-5 flex items-center justify-center rounded opacity-0 group-hover:opacity-100 hover:bg-red-50 dark:hover:bg-red-900/30 text-slate-300 hover:text-red-500 transition-all"
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </div>
            <div className="flex items-center gap-2 mt-1">
              <span className="text-[11px] text-slate-400 dark:text-gray-500">
                {t(messages.vars_count, { n: group.variable_count })}
              </span>
              {group.priority > 0 && (
                <span className="text-[11px] px-1.5 py-0.5 rounded bg-slate-100 dark:bg-gray-700 text-slate-400 dark:text-gray-500">
                  P{group.priority}
                </span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}
