import { AppRoot } from "@/components/app-root";
import { PageShell } from "@/components/page-shell";
import { ROUTES } from "@/utils/routes";

export function Home() {
  return (
    <AppRoot>
      <PageShell title="Exam Platform" path={ROUTES.home}>
        {null}
      </PageShell>
    </AppRoot>
  );
}
