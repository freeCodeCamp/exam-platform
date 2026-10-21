import { useDraggable } from "@dnd-kit/react";
import { Badge } from "@chakra-ui/react";

export function Draggable({ id }: { id: string }) {
  const { ref } = useDraggable({
    id,
  });

  return <Badge ref={ref}> {id} </Badge>;
}
