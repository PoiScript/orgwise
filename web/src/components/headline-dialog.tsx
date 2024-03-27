import {
  Calendar,
  CheckCircle2,
  Circle,
  CircleDashed,
  Flag,
  Tags,
} from "lucide-react";
import { ReactNode } from "react";
import {
  Control,
  Controller,
  ControllerRenderProps,
  FieldPath,
  UseFormStateReturn,
  useForm,
} from "react-hook-form";

import { SearchResult, commandAtom } from "@/atom";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useSetAtom } from "jotai";

export function DropdownMenuDemo({
  item,
  onClose,
}: {
  item: SearchResult;
  onClose: VoidFunction;
}) {
  const { control, register, handleSubmit } = useForm<SearchResult>({
    defaultValues: item,
  });

  const executeCommand = useSetAtom(commandAtom);

  return (
    <form
      onSubmit={handleSubmit(async (value) => {
        await executeCommand("update-headline", {
          url: value.url,
          line: value.line,
          keyword: value.keyword?.value || "",
          priority: value.priority || "",
          title: value.title || "",
          section: value.section || "",
        });
        onClose();
      })}
    >
      <div className="p-4 flex flex-row items-center justify-center gap-2">
        <label htmlFor="title" className="sr-only">
          Title
        </label>
        <input
          type="text"
          name="title"
          id="title"
          className="text-lg font-medium border-0 flex-1 outline-0 block"
          placeholder="Title"
          {...register("title", { required: true })}
        />
      </div>

      <label htmlFor="section" className="sr-only">
        Section
      </label>
      <textarea
        rows={5}
        name="section"
        id="section"
        className="px-4 text-base border-0 w-full outline-0 block resize-none hover:resize"
        placeholder="Write section..."
        {...register("section")}
      />

      <div className="px-4 py-2 flex items-center justify-end gap-x-2">
        <ControlledDropdown
          control={control}
          name="keyword"
          renderContent={(field) => (
            <>
              {["TODO"].map((i) => (
                <DropdownMenuCheckboxItem
                  key={i}
                  checked={field.value && field.value.value == i}
                  onCheckedChange={(checked) =>
                    field.onChange(checked ? { value: i, type: "TODO" } : null)
                  }
                >
                  {i}
                </DropdownMenuCheckboxItem>
              ))}
              {["DONE"].map((i) => (
                <DropdownMenuCheckboxItem
                  key={i}
                  checked={field.value && field.value.value == i}
                  onCheckedChange={(checked) =>
                    field.onChange(checked ? { value: i, type: "DONE" } : null)
                  }
                >
                  {i}
                </DropdownMenuCheckboxItem>
              ))}
            </>
          )}
          renderTrigger={(field, formState) => (
            <Button
              size="sm"
              className="rounded-full border-0 gap-1"
              variant={field.value ? "default" : "secondary"}
              disabled={formState.isLoading}
            >
              {field.value ? (
                field.value.type === "TODO" ? (
                  <Circle size={18} />
                ) : (
                  <CheckCircle2 size={18} />
                )
              ) : (
                <CircleDashed size={18} />
              )}
              Status
            </Button>
          )}
        />

        <ControlledDropdown
          control={control}
          name="priority"
          renderContent={(field) => (
            <>
              {["A", "B", "C"].map((i) => (
                <DropdownMenuCheckboxItem
                  checked={field.value == i}
                  onCheckedChange={(checked) =>
                    field.onChange(checked ? i : null)
                  }
                >
                  #{i}
                </DropdownMenuCheckboxItem>
              ))}
            </>
          )}
          renderTrigger={(field, formState) => (
            <Button
              size="sm"
              className="rounded-full border-0 gap-1"
              variant={field.value ? "default" : "secondary"}
              disabled={formState.isLoading}
            >
              <Flag size={18} />
              {field.value ? `#${field.value}` : "Priority"}
            </Button>
          )}
        />

        <ControlledDropdown
          control={control}
          name="tags"
          renderContent={(field) => (
            <>
              {["foo", "bar"].map((i) => (
                <DropdownMenuCheckboxItem
                  // checked={field.value == i}
                  onCheckedChange={(checked) =>
                    field.onChange(checked ? i : null)
                  }
                >
                  {i}
                </DropdownMenuCheckboxItem>
              ))}
            </>
          )}
          renderTrigger={(field, formState) => (
            <Button
              size="sm"
              className="rounded-full border-0 gap-1"
              variant={
                field.value && field.value.length > 0 ? "default" : "secondary"
              }
              disabled={formState.isLoading}
            >
              <Tags size={18} />
              {field.value && field.value.length > 0 ? (
                <>
                  {field.value[0]}
                  {field.value.length > 1 && ` (+${field.value.length + 1})`}
                </>
              ) : (
                "Tags"
              )}
            </Button>
          )}
        />
      </div>

      <div className="p-2 border-t-2 flex justify-between items-center">
        <div>
          <Button
            size="sm"
            className="text-red-500"
            variant="ghost"
            type="button"
            onClick={async () => {
              await executeCommand("remove-headline", {
                url: item.url,
                line: item.line,
              });
              onClose();
            }}
          >
            Delete
          </Button>

          <Button
            size="sm"
            variant="ghost"
            type="button"
            onClick={async () => {
              await executeCommand("duplicate-headline", {
                url: item.url,
                line: item.line,
              });
              onClose();
            }}
          >
            Duplicate
          </Button>
        </div>

        <Button size="sm" variant="ghost" type="submit">
          Save
        </Button>
      </div>
    </form>
  );
}

function ControlledDropdown<
  TFieldValues,
  TName extends FieldPath<TFieldValues>,
>({
  name,
  control,
  renderTrigger,
  renderContent,
}: {
  name: TName;
  control: Control<TFieldValues>;
  renderTrigger: (
    filed: ControllerRenderProps<TFieldValues, TName>,
    formState: UseFormStateReturn<TFieldValues>
  ) => ReactNode;
  renderContent: (
    filed: ControllerRenderProps<TFieldValues, TName>,
    formState: UseFormStateReturn<TFieldValues>
  ) => ReactNode;
}) {
  return (
    <Controller<TFieldValues, TName>
      control={control}
      name={name}
      render={({ field, formState }) => (
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            {renderTrigger(field, formState)}
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {renderContent(field, formState)}
          </DropdownMenuContent>
        </DropdownMenu>
      )}
    />
  );
}
