import { Moon, Sun } from "lucide-react";
import { Button } from "./button";
import { useTheme } from "../../context/ThemeContext";

export function ModeToggle() {
  const { theme, resolvedTheme, setTheme } = useTheme();

  const toggleTheme = () => {
    if (resolvedTheme === "dark") {
      setTheme("light");
    } else {
      setTheme("dark");
    }
  };

  return (
    <Button
      variant="ghost"
      size="sm"
      onClick={toggleTheme}
      title={`Switch to ${resolvedTheme === "dark" ? "Light" : "Dark"} mode (current: ${theme})`}
      className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground"
    >
      {resolvedTheme === "dark" ? (
        <Sun className="size-4 text-amber-400 transition-transform duration-200" />
      ) : (
        <Moon className="size-4 text-slate-700 transition-transform duration-200" />
      )}
      <span className="sr-only">Toggle theme</span>
    </Button>
  );
}
