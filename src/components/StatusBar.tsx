interface GroupInfo {
  name: string;
  active: boolean;
}

interface Props {
  groups: GroupInfo[];
}

export function StatusBar({ groups }: Props) {
  const activeCount = groups.filter((g) => g.active).length;
  const totalCount = groups.length;

  return (
    <footer className="bg-gray-100 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-2 flex items-center justify-between text-xs text-gray-500">
      <span>
        {activeCount} active / {totalCount} total groups
      </span>
      <span>Shell hooks auto-reload on change</span>
    </footer>
  );
}
