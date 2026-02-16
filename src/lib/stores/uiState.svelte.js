class UIState {
  contextMenu = $state({
    visible: false,
    x: 0,
    y: 0,
    item: null, // File/Folder item data
    actions: []
  });

  dialog = $state({
    visible: false,
    title: '',
    message: '',
    confirmLabel: 'Confirm',
    cancelLabel: 'Cancel',
    onConfirm: null,
    onCancel: null,
    variant: 'default' // 'default', 'danger'
  });

  draggedItem = $state(null);

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

  showDialog(config) {
    this.dialog = {
      visible: true,
      title: config.title || 'Are you sure?',
      message: config.message || '',
      confirmLabel: config.confirmLabel || 'Confirm',
      cancelLabel: config.cancelLabel || 'Cancel',
      onConfirm: config.onConfirm || null,
      onCancel: config.onCancel || null,
      variant: config.variant || 'default'
    };
  }

  closeDialog() {
    this.dialog.visible = false;
  }
}

export const uiState = new UIState();
