import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";

export function Home() {
  return (
    <AppRoot>
      <PageShell
        title="Examiner Dashboard"
        description="Author, review, promote, and moderate exams."
        path="/"
      >
        {null}
      </PageShell>
    </AppRoot>
  );
}
