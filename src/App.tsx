import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { GroupList } from "./components/GroupList";
import { GroupEditor } from "./components/GroupEditor";
import { StatusBar } from "./components/StatusBar";
import { CreateGroupModal } from "./components/CreateGroupModal";

interface GroupInfo {
  name: string;
  description: string;
  active: boolean;
  priority: number;
  variable_count: number;
}

export default function App() {
  const [groups, setGroups] = useState<GroupInfo[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadGroups = useCallback(async () => {
    try {
      const result = await invoke<GroupInfo[]>("get_groups");
      setGroups(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    loadGroups();
  }, [loadGroups]);

  const handleToggle = async (name: string, active: boolean) => {
    try {
      if (active) {
        await invoke("disable_group", { name });
      } else {
        await invoke("enable_group", { name });
      }
      await loadGroups();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDelete = async (name: string) => {
    try {
      await invoke("delete_group", { name });
      if (selectedGroup === name) setSelectedGroup(null);
      await loadGroups();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleCreate = async (name: string, description: string, priority: number) => {
    try {
      await invoke("create_group", { name, description, priority });
      setShowCreateModal(false);
      await loadGroups();
      setSelectedGroup(name);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="h-screen flex flex-col">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-3 flex items-center justify-between">
        <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
          EnvTools
        </h1>
        <button
          onClick={() => setShowCreateModal(true)}
          className="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 transition-colors"
        >
          + New Group
        </button>
      </header>

      {/* Error Banner */}
      {error && (
        <div className="bg-red-50 border-b border-red-200 px-6 py-2 text-sm text-red-700 flex justify-between items-center">
          <span>{error}</span>
          <button onClick={() => setError(null)} className="text-red-500 hover:text-red-700">
            &times;
          </button>
        </div>
      )}

      {/* Main Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar - Group List */}
        <aside className="w-72 border-r border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 overflow-y-auto">
          <GroupList
            groups={groups}
            selectedGroup={selectedGroup}
            onSelect={setSelectedGroup}
            onToggle={handleToggle}
            onDelete={handleDelete}
          />
        </aside>

        {/* Main Panel - Group Editor */}
        <main className="flex-1 overflow-y-auto bg-gray-50 dark:bg-gray-900">
          {selectedGroup ? (
            <GroupEditor groupName={selectedGroup} onUpdate={loadGroups} />
          ) : (
            <div className="flex items-center justify-center h-full text-gray-400">
              <p>Select a group to edit variables</p>
            </div>
          )}
        </main>
      </div>

      {/* Status Bar */}
      <StatusBar groups={groups} />

      {/* Create Modal */}
      {showCreateModal && (
        <CreateGroupModal
          onClose={() => setShowCreateModal(false)}
          onCreate={handleCreate}
        />
      )}
    </div>
  );
}
