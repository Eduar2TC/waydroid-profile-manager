import 'package:flutter/material.dart';

import '../../domain/entities/profile.dart';
import '../../domain/enums/orientation.dart';

/// Result of the profile editor dialog.
class ProfileEditorResult {
  final String name;
  final String? description;
  final int baseWidth;
  final int baseHeight;
  final int density;
  final WaydroidOrientation defaultOrientation;

  const ProfileEditorResult({
    required this.name,
    this.description,
    required this.baseWidth,
    required this.baseHeight,
    required this.density,
    required this.defaultOrientation,
  });
}

class ProfileEditorDialog extends StatefulWidget {
  final Profile? profile;

  const ProfileEditorDialog({super.key, this.profile});

  @override
  State<ProfileEditorDialog> createState() => _ProfileEditorDialogState();
}

class _ProfileEditorDialogState extends State<ProfileEditorDialog> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _nameController;
  late final TextEditingController _descriptionController;
  late final TextEditingController _widthController;
  late final TextEditingController _heightController;
  late final TextEditingController _densityController;
  late WaydroidOrientation _orientation;

  bool get isEditing => widget.profile != null;

  @override
  void initState() {
    super.initState();
    final p = widget.profile;
    _nameController = TextEditingController(text: p?.name ?? '');
    _descriptionController = TextEditingController(text: p?.description ?? '');
    _widthController = TextEditingController(
      text: p?.baseWidth.toString() ?? '',
    );
    _heightController = TextEditingController(
      text: p?.baseHeight.toString() ?? '',
    );
    _densityController = TextEditingController(
      text: p?.density.toString() ?? '',
    );
    _orientation = p?.defaultOrientation ?? WaydroidOrientation.portrait;
  }

  @override
  void dispose() {
    _nameController.dispose();
    _descriptionController.dispose();
    _widthController.dispose();
    _heightController.dispose();
    _densityController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: Text(isEditing ? 'Edit Profile' : 'Create Profile'),
      content: SizedBox(
        width: 450,
        child: Form(
          key: _formKey,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                // Name
                TextFormField(
                  controller: _nameController,
                  decoration: const InputDecoration(
                    labelText: 'Name *',
                    hintText: 'e.g., Pixel 7',
                    border: OutlineInputBorder(),
                  ),
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return 'Name is required';
                    }
                    if (value.length > 100) {
                      return 'Name too long';
                    }
                    return null;
                  },
                ),
                const SizedBox(height: 16),

                // Description
                TextFormField(
                  controller: _descriptionController,
                  decoration: const InputDecoration(
                    labelText: 'Description',
                    hintText: 'Optional description',
                    border: OutlineInputBorder(),
                  ),
                  maxLines: 2,
                ),
                const SizedBox(height: 16),

                // Width and Height side by side
                Row(
                  children: [
                    Expanded(
                      child: TextFormField(
                        controller: _widthController,
                        decoration: const InputDecoration(
                          labelText: 'Width (base) *',
                          hintText: '640',
                          border: OutlineInputBorder(),
                        ),
                        keyboardType: TextInputType.number,
                        validator: _validateDimension,
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: TextFormField(
                        controller: _heightController,
                        decoration: const InputDecoration(
                          labelText: 'Height (base) *',
                          hintText: '360',
                          border: OutlineInputBorder(),
                        ),
                        keyboardType: TextInputType.number,
                        validator: _validateDimension,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 16),

                // Density
                TextFormField(
                  controller: _densityController,
                  decoration: const InputDecoration(
                    labelText: 'DPI (density) *',
                    hintText: '160',
                    border: OutlineInputBorder(),
                    helperText: 'Range: 40–1000',
                  ),
                  keyboardType: TextInputType.number,
                  validator: (value) {
                    if (value == null || value.trim().isEmpty) {
                      return 'Density is required';
                    }
                    final d = int.tryParse(value.trim());
                    if (d == null || d < 40 || d > 1000) {
                      return 'Must be between 40 and 1000';
                    }
                    return null;
                  },
                ),
                const SizedBox(height: 16),

                // Default Orientation
                const Text(
                  'Default Orientation',
                  style: TextStyle(fontSize: 13, color: Colors.white70),
                ),
                const SizedBox(height: 8),
                SegmentedButton<WaydroidOrientation>(
                  segments: const [
                    ButtonSegment(
                      value: WaydroidOrientation.landscape,
                      label: Text('Landscape'),
                      icon: Icon(Icons.screen_rotation, size: 18),
                    ),
                    ButtonSegment(
                      value: WaydroidOrientation.portrait,
                      label: Text('Portrait'),
                      icon: Icon(Icons.screen_lock_rotation, size: 18),
                    ),
                  ],
                  selected: {_orientation},
                  onSelectionChanged: (set) {
                    setState(() => _orientation = set.first);
                  },
                ),
                const SizedBox(height: 16),

                // Preview
                _buildPreview(),
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _save,
          child: Text(isEditing ? 'Save' : 'Create'),
        ),
      ],
    );
  }

  Widget _buildPreview() {
    final width = int.tryParse(_widthController.text) ?? 0;
    final height = int.tryParse(_heightController.text) ?? 0;
    final density = int.tryParse(_densityController.text) ?? 0;

    if (width == 0 || height == 0) {
      return const SizedBox.shrink();
    }

    // Calculate display ratio
    final maxPreviewWidth = 150.0;
    final maxPreviewHeight = 200.0;
    final aspectRatio = width / height;

    double previewWidth;
    double previewHeight;
    if (aspectRatio > 1) {
      previewWidth = maxPreviewWidth;
      previewHeight = maxPreviewWidth / aspectRatio;
    } else {
      previewHeight = maxPreviewHeight;
      previewWidth = maxPreviewHeight * aspectRatio;
    }

    // Ensure within bounds
    if (previewWidth > maxPreviewWidth) {
      previewWidth = maxPreviewWidth;
      previewHeight = previewWidth / aspectRatio;
    }
    if (previewHeight > maxPreviewHeight) {
      previewHeight = maxPreviewHeight;
      previewWidth = previewHeight * aspectRatio;
    }

    final dims = _orientation.toActual(width, height);

    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white.withValues(alpha: 0.03),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: Colors.white.withValues(alpha: 0.1)),
      ),
      child: Column(
        children: [
          const Text(
            'Preview',
            style: TextStyle(fontSize: 12, color: Colors.white54),
          ),
          const SizedBox(height: 8),
          Container(
            width: previewWidth,
            height: previewHeight,
            decoration: BoxDecoration(
              color: Colors.white.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5),
              ),
            ),
            child: Center(
              child: Text(
                '${dims[0]}×${dims[1]}',
                style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w500),
              ),
            ),
          ),
          const SizedBox(height: 8),
          Text(
            '$density DPI • ${_orientation.displayName}',
            style: TextStyle(
              fontSize: 12,
              color: Colors.white.withValues(alpha: 0.6),
            ),
          ),
        ],
      ),
    );
  }

  String? _validateDimension(String? value) {
    if (value == null || value.trim().isEmpty) {
      return 'Required';
    }
    final d = int.tryParse(value.trim());
    if (d == null || d < 100 || d > 10000) {
      return 'Must be 100–10000';
    }
    return null;
  }

  void _save() {
    if (!_formKey.currentState!.validate()) return;

    final result = ProfileEditorResult(
      name: _nameController.text.trim(),
      description: _descriptionController.text.trim().isEmpty
          ? null
          : _descriptionController.text.trim(),
      baseWidth: int.parse(_widthController.text.trim()),
      baseHeight: int.parse(_heightController.text.trim()),
      density: int.parse(_densityController.text.trim()),
      defaultOrientation: _orientation,
    );

    Navigator.of(context).pop(result);
  }
}
