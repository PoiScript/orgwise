import { Action, ActionPanel, Form } from "@raycast/api";
import { parse, lightFormat } from "date-fns";
import { useState } from "react";
import { useAtomValue } from "jotai";
import {
  orgDoneKeywordsAtom,
  orgPrioritiesAtom,
  orgTagsAtom,
  orgTodoKeywordsAtom,
} from "./atom";

import type { SearchResult } from "../../web/src/atom";

const formatStr = "yyyy-MM-dd'T'HH:mm:ss";

export const TaskForm: React.FC<{
  defaultValue: Partial<SearchResult>;
  onSubmit: (values: any) => void;
}> = ({ defaultValue, onSubmit }) => {
  const [titleError, setTitleError] = useState<string | undefined>();

  const todoKeywords = useAtomValue(orgTodoKeywordsAtom);
  const doneKeywords = useAtomValue(orgDoneKeywordsAtom);
  const tags = useAtomValue(orgTagsAtom);
  const priorities = useAtomValue(orgPrioritiesAtom);

  function dropTitleErrorIfNeeded() {
    if (titleError && titleError.length > 0) {
      setTitleError(undefined);
    }
  }

  return (
    <Form
      actions={
        <ActionPanel>
          <Action.SubmitForm
            title="Submit"
            onSubmit={(values) => {
              onSubmit({
                ...values,

                scheduled: values.scheduled
                  ? lightFormat(values.scheduled, formatStr)
                  : null,

                deadline: values.deadline
                  ? lightFormat(values.deadline, formatStr)
                  : null,
              });
            }}
          />
        </ActionPanel>
      }
    >
      <Form.TextField
        id="title"
        title="Title"
        placeholder="Enter title"
        error={titleError}
        onChange={dropTitleErrorIfNeeded}
        defaultValue={defaultValue.title}
        onBlur={(event) => {
          if (event.target.value?.length == 0) {
            setTitleError("The field should't be empty!");
          } else {
            dropTitleErrorIfNeeded();
          }
        }}
      />

      <Form.Dropdown
        id="keyword"
        title="Status"
        defaultValue={defaultValue.keyword?.value}
      >
        <Form.Dropdown.Section title="TODO">
          {todoKeywords.map((t) => (
            <Form.Dropdown.Item key={t} value={t} title={t} />
          ))}
        </Form.Dropdown.Section>
        <Form.Dropdown.Section title="DONE">
          {doneKeywords.map((t) => (
            <Form.Dropdown.Item key={t} value={t} title={t} />
          ))}
        </Form.Dropdown.Section>
      </Form.Dropdown>

      <Form.Dropdown
        id="priority"
        title="Priority"
        defaultValue={defaultValue.priority}
        placeholder="Add priority"
      >
        {priorities.map((p) => (
          <Form.Dropdown.Item key={p} value={p} title={"#" + p} />
        ))}
      </Form.Dropdown>

      <Form.TagPicker
        id="tags"
        title="Tags"
        defaultValue={defaultValue.tags}
        placeholder="Add tags"
      >
        {tags.map((t) => (
          <Form.TagPicker.Item key={t} value={t} title={t} />
        ))}
      </Form.TagPicker>

      <Form.TextArea
        id="section"
        title="Section"
        defaultValue={defaultValue.section}
        placeholder="Enter section"
      />

      <Form.DatePicker
        id="scheduled"
        title="Schedule"
        defaultValue={
          defaultValue?.planning?.scheduled
            ? parse(defaultValue.planning.scheduled, formatStr, new Date())
            : null
        }
      />

      <Form.DatePicker
        id="deadline"
        title="Deadline"
        defaultValue={
          defaultValue?.planning?.deadline
            ? parse(defaultValue.planning.deadline, formatStr, new Date())
            : null
        }
      />
    </Form>
  );
};
