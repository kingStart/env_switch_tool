import { useState } from "react";
import { useI18n } from "../i18n";

interface Props {
  onClose: () => void;
  onCreate: (name: string, description: string, priority: number) => void;
}

export function CreateGroupModal({ onClose, onCreate }: Props) {
  const { messages } = useI18n();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState(0);

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) return;
    onCreate(name.trim(), description, priority);
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/30 backdrop-blur-sm" onClick={onClose} />
      <div className="relative bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
        <div className="px-6 py-5 border-b border-slate-100 dark:border-gray-700">
          <h3 className="text-lg font-bold text-slate-800 dark:text-white">{messages.create_group}</h3>
        </div>
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-gray-300 mb-1.5">
              {messages.group_name}
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={messages.group_name_placeholder}
              autoFocus
              className="w-full px-3 py-2.5 border border-slate-200 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-xl text-sm text-slate-800 dark:text-gray-200 placeholder:text-slate-300 dark:placeholder:text-gray-500 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-gray-300 mb-1.5">
              {messages.description}
              <span className="text-slate-300 dark:text-gray-500 font-normal ml-1">(optional)</span>
            </label>
            <input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={messages.description_placeholder}
              className="w-full px-3 py-2.5 border border-slate-200 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-xl text-sm text-slate-800 dark:text-gray-200 placeholder:text-slate-300 dark:placeholder:text-gray-500 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-gray-300 mb-1.5">
              {messages.priority}
              <span className="text-slate-300 dark:text-gray-500 font-normal ml-1">({messages.priority_hint})</span>
            </label>
            <input
              type="number"
              value={priority}
              onChange={(e) => setPriority(Number(e.target.value))}
              min={0}
              className="w-24 px-3 py-2.5 border border-slate-200 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-xl text-sm text-slate-800 dark:text-gray-200 focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-shadow"
            />
          </div>
          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-slate-600 dark:text-gray-400 hover:bg-slate-50 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              {messages.cancel}
            </button>
            <button
              type="submit"
              className="px-5 py-2 bg-indigo-500 hover:bg-indigo-600 text-white text-sm font-medium rounded-lg transition-colors shadow-sm"
            >
              {messages.create}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
