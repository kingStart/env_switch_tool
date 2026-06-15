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
  if (groups.length === 0) {
    return (
      <div className="p-6 text-center text-gray-400 text-sm">
        No groups yet. Click "+ New Group" to create one.
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-100 dark:divide-gray-700">
      {groups.map((group) => (
        <div
          key={group.name}
          className={`p-4 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors ${
            selectedGroup === group.name ? "bg-blue-50 dark:bg-blue-900/20 border-l-3 border-blue-500" : ""
          }`}
          onClick={() => onSelect(group.name)}
        >
          <div className="flex items-center justify-between mb-1">
            <span className="font-medium text-sm text-gray-900 dark:text-white truncate">
              {group.name}
            </span>
            <div className="flex items-center gap-2">
              {/* Toggle Switch */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onToggle(group.name, group.active);
                }}
                className={`relative w-9 h-5 rounded-full transition-colors ${
                  group.active ? "bg-green-500" : "bg-gray-300"
                }`}
              >
                <span
                  className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full transition-transform ${
                    group.active ? "translate-x-4" : ""
                  }`}
                />
              </button>
              {/* Delete */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  if (confirm(`Delete group "${group.name}"?`)) {
                    onDelete(group.name);
                  }
                }}
                className="text-gray-400 hover:text-red-500 text-xs"
                title="Delete"
              >
                &times;
              </button>
            </div>
          </div>
          <p className="text-xs text-gray-500 truncate">{group.description || "No description"}</p>
          <div className="mt-1 flex gap-3 text-xs text-gray-400">
            <span>{group.variable_count} vars</span>
            <span>priority: {group.priority}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
