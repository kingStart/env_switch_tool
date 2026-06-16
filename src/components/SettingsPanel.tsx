import { useI18n } from "../i18n";
import type { Theme } from "../hooks/useTheme";

interface Props {
  theme: Theme;
  setTheme: (t: Theme) => void;
}

export function SettingsPanel({ theme, setTheme }: Props) {
  const { messages, locale, setLocale } = useI18n();

  const themeOptions: { value: Theme; label: string }[] = [
    { value: "system", label: messages.theme_system },
    { value: "light", label: messages.theme_light },
    { value: "dark", label: messages.theme_dark },
  ];

  const langOptions = [
    { value: "en" as const, label: "English" },
    { value: "zh" as const, label: "中文" },
  ];

  return (
    <div className="p-4 space-y-5">
      {/* Theme */}
      <div>
        <label className="block text-xs font-semibold text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-2">
          {messages.theme}
        </label>
        <div className="flex rounded-lg overflow-hidden border border-slate-200 dark:border-gray-600">
          {themeOptions.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setTheme(opt.value)}
              className={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
                theme === opt.value
                  ? "bg-indigo-500 text-white"
                  : "bg-white dark:bg-gray-700 text-slate-600 dark:text-gray-300 hover:bg-slate-50 dark:hover:bg-gray-600"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* Language */}
      <div>
        <label className="block text-xs font-semibold text-slate-500 dark:text-gray-400 uppercase tracking-wider mb-2">
          {messages.language}
        </label>
        <div className="flex rounded-lg overflow-hidden border border-slate-200 dark:border-gray-600">
          {langOptions.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setLocale(opt.value)}
              className={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
                locale === opt.value
                  ? "bg-indigo-500 text-white"
                  : "bg-white dark:bg-gray-700 text-slate-600 dark:text-gray-300 hover:bg-slate-50 dark:hover:bg-gray-600"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
