import { useDraggable } from "@dnd-kit/react";
import { Feedback } from "@dnd-kit/dom";
import { Badge } from "@chakra-ui/react";

export function Draggable({ id }: { id: string }) {
  const { ref } = useDraggable({
    id,
    plugins: [Feedback.configure({ feedback: "clone", dropAnimation: null })],
  });

  return <Badge ref={ref}> {id} </Badge>;
}
