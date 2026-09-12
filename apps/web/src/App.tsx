import { api } from "./api";
import {
  ThemeProvider,
  AuthProvider,
  LedgerProvider,
  ModalProvider,
  DashboardLayout,
} from "@sealed-books/ui";

export default function App() {
  return (
    <ThemeProvider defaultTheme="system">
      <AuthProvider api={api}>
        <LedgerProvider api={api}>
          <ModalProvider>
            <DashboardLayout />
          </ModalProvider>
        </LedgerProvider>
      </AuthProvider>
    </ThemeProvider>
  );
}
