#include "my_application.h"

#include <flutter_linux/flutter_linux.h>

#include <unistd.h>
#ifdef GDK_WINDOWING_X11
#include <gdk/gdkx.h>
#endif

#include "flutter/generated_plugin_registrant.h"

// `decoration`/`decoration:backdrop` round the window's outer frame/shadow.
// `window.background` rounds the actual painted background behind the
// content and gives it the app's real background color — this is the node
// that must be rounded for the corners to look right where FlView (which
// renders via its own GL surface and does NOT respect Flutter-side clips
// for its base fill) shows through.
static const char* css_style =
    "decoration, decoration:backdrop {"
    "  border-radius: 12px;"
    "  box-shadow: none;"
    "  border: none;"
    "}"
    "window.background {"
    "  border-radius: 12px;"
    "  background-color: #1C1B1F;"
    "}";

struct _MyApplication {
  GtkApplication parent_instance;
  char** dart_entrypoint_arguments;
};

G_DEFINE_TYPE(MyApplication, my_application, GTK_TYPE_APPLICATION)

// Absolute path of the app icon inside the running bundle, resolved from
// /proc/self/exe so it works no matter where the bundle lives. Returns NULL
// when the icon is not shipped (e.g. an older bundle).
static gchar* bundle_icon_path(void) {
  char exe[4096];
  ssize_t len = readlink("/proc/self/exe", exe, sizeof(exe) - 1);
  if (len <= 0) return nullptr;
  exe[len] = '\0';
  g_autofree gchar* dir = g_path_get_dirname(exe);
  gchar* path = g_build_filename(dir, "data", "flutter_assets", "assets",
                                 "icon.png", nullptr);
  if (!g_file_test(path, G_FILE_TEST_IS_REGULAR)) {
    g_clear_pointer(&path, g_free);
    return nullptr;
  }
  return path;
}

// Called when first Flutter frame received.
static void first_frame_cb(MyApplication* self, FlView* view) {
  gtk_widget_show(gtk_widget_get_toplevel(GTK_WIDGET(view)));
}

// Implements GApplication::activate.
static void my_application_activate(GApplication* application) {
  MyApplication* self = MY_APPLICATION(application);
  GtkWindow* window =
      GTK_WINDOW(gtk_application_window_new(GTK_APPLICATION(application)));

  // Window icon (titlebar, Alt-Tab, taskbar) from the bundled icon. The
  // launcher/menu icon is a separate thing: it comes from the .desktop file
  // and the hicolor icons shipped by the release (StartupWMClass below must
  // match APPLICATION_ID so both refer to the same window).
  g_autofree gchar* icon_path = bundle_icon_path();
  if (icon_path != nullptr) {
    g_autoptr(GError) icon_error = nullptr;
    if (!gtk_window_set_icon_from_file(window, icon_path, &icon_error)) {
      g_warning("Failed to load window icon %s: %s", icon_path,
                icon_error->message);
    }
  }

  // Do NOT set a CSD header bar here. KWin (KDE Plasma) always draws its own
  // server-side title bar for this window, and forcing a GTK header bar made
  // TWO title bars stack up (icon + min/max/close duplicated). Letting the
  // window manager provide the title bar yields exactly one bar with the app
  // icon, and the rounded corners still come from `window.background` in the
  // CSS below (rounded at the bottom, square at the top where the title bar
  // sits — the KDE-native look). The window title is what KWin shows there.
  gtk_window_set_title(window, "Waydroid Profile Manager");
  gtk_window_set_default_size(window, 500, 700);

  GdkGeometry hints;
  hints.min_width = 400;
  hints.min_height = 500;
  hints.max_width = 500;
  hints.max_height = 700;
  gtk_window_set_geometry_hints(window, nullptr, &hints,
      static_cast<GdkWindowHints>(GDK_HINT_MIN_SIZE | GDK_HINT_MAX_SIZE));

  GdkScreen* screen = gtk_window_get_screen(window);

  // RGBA visual so the corners GTK leaves unpainted (outside the rounded
  // `window.background`) are truly transparent, not solid black.
  gboolean have_compositor = gdk_screen_is_composited(screen);
  if (have_compositor) {
    GdkVisual* visual = gdk_screen_get_rgba_visual(screen);
    if (visual != nullptr) {
      gtk_widget_set_visual(GTK_WIDGET(window), visual);
    } else {
      have_compositor = FALSE;
      g_warning("RGBA visual unavailable; rounded corners will not render "
                "correctly on this display.");
    }
  } else {
    g_warning("No compositor detected; skipping RGBA visual. Rounded "
              "corners require a running compositor.");
  }

  g_autoptr(FlDartProject) project = fl_dart_project_new();
  fl_dart_project_set_dart_entrypoint_arguments(
      project, self->dart_entrypoint_arguments);

  FlView* view = fl_view_new(project);

  // IMPORTANT: transparent background on FlView itself. FlView renders via
  // its own GL/EGL surface, which is NOT clipped by Flutter-side widgets
  // like ClipRRect (that only affects what Flutter paints, not FlView's
  // base fill) and is NOT clipped by GTK CSS either (it isn't a normal
  // cairo-drawn widget). If this stays opaque, it paints a full rectangle
  // that covers any rounding you do elsewhere — which is exactly the
  // square-corner artifact you were seeing. The real background color now
  // lives on `window.background` in the CSS above, which IS a normal
  // cairo-drawn, CSS-clippable node.
  GdkRGBA transparent = {0.0, 0.0, 0.0, 0.0};
  fl_view_set_background_color(view, &transparent);

  gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));

  GtkCssProvider* css_provider = gtk_css_provider_new();
  gtk_css_provider_load_from_data(css_provider, css_style, -1, nullptr);
  gtk_style_context_add_provider_for_screen(
      screen,
      GTK_STYLE_PROVIDER(css_provider),
      GTK_STYLE_PROVIDER_PRIORITY_USER);
  g_object_unref(css_provider);

  fl_register_plugins(FL_PLUGIN_REGISTRY(view));

  gtk_widget_show(GTK_WIDGET(view));

  g_signal_connect_swapped(view, "first-frame", G_CALLBACK(first_frame_cb),
                           self);
  gtk_widget_realize(GTK_WIDGET(view));

  gtk_widget_grab_focus(GTK_WIDGET(view));
}

// Implements GApplication::local_command_line.
static gboolean my_application_local_command_line(GApplication* application,
                                                  gchar*** arguments,
                                                  int* exit_status) {
  MyApplication* self = MY_APPLICATION(application);
  self->dart_entrypoint_arguments = g_strdupv(*arguments + 1);

  g_autoptr(GError) error = nullptr;
  if (!g_application_register(application, nullptr, &error)) {
    g_warning("Failed to register: %s", error->message);
    *exit_status = 1;
    return TRUE;
  }

  g_application_activate(application);
  *exit_status = 0;

  return TRUE;
}

// Implements GApplication::startup.
static void my_application_startup(GApplication* application) {
  G_APPLICATION_CLASS(my_application_parent_class)->startup(application);
}

// Implements GApplication::shutdown.
static void my_application_shutdown(GApplication* application) {
  G_APPLICATION_CLASS(my_application_parent_class)->shutdown(application);
}

// Implements GObject::dispose.
static void my_application_dispose(GObject* object) {
  MyApplication* self = MY_APPLICATION(object);
  g_clear_pointer(&self->dart_entrypoint_arguments, g_strfreev);
  G_OBJECT_CLASS(my_application_parent_class)->dispose(object);
}

static void my_application_class_init(MyApplicationClass* klass) {
  G_APPLICATION_CLASS(klass)->activate = my_application_activate;
  G_APPLICATION_CLASS(klass)->local_command_line =
      my_application_local_command_line;
  G_APPLICATION_CLASS(klass)->startup = my_application_startup;
  G_APPLICATION_CLASS(klass)->shutdown = my_application_shutdown;
  G_OBJECT_CLASS(klass)->dispose = my_application_dispose;
}

static void my_application_init(MyApplication* self) {}

MyApplication* my_application_new() {
  g_set_prgname(APPLICATION_ID);

  return MY_APPLICATION(g_object_new(my_application_get_type(),
                                     "application-id", APPLICATION_ID, "flags",
                                     G_APPLICATION_NON_UNIQUE, nullptr));
}