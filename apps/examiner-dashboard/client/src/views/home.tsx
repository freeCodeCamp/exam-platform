import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ProtectedRoute } from "@/components/protected-route";

export function Home() {
  return (
    <AppRoot>
      <ProtectedRoute>
        <PageShell
          title="Examiner Dashboard"
          description="Author, review, promote, and moderate exams."
          path="/"
        >
          {null}
        </PageShell>
      </ProtectedRoute>
    </AppRoot>
  );
}
