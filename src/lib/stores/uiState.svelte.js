class UIState {
  contextMenu = $state({
    visible: false,
    x: 0,
    y: 0,
    item: null, // File/Folder item data
    actions: []
  });

  editorStatus = $state({
    line: 1,
    column: 1,
    selectionCount: 0,
    language: 'Plain Text'
  });

  updateEditorStatus(status) {
    this.editorStatus = { ...this.editorStatus, ...status };
  }

  openContextMenu(e, item, actions) {
    e.preventDefault();
    this.contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      item,
      actions
    };
  }

  closeContextMenu() {
    this.contextMenu.visible = false;
  }
}

export const uiState = new UIState();
