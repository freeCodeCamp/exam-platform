import { useDroppable } from "@dnd-kit/react";

export function Droppable({ id, children }: { id: string; children: React.ReactNode }) {
  const { ref } = useDroppable({
    id,
  });

  return (
    <div ref={ref} style={{ width: 300, height: 300 }}>
      {children}
    </div>
  );
}
