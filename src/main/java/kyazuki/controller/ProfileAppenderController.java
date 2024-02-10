package kyazuki.controller;

import java.io.FileInputStream;
import java.io.FileWriter;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.UUID;
import java.util.logging.Level;
import java.util.logging.Logger;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

import javafx.concurrent.Service;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.DialogEvent;
import kyazuki.App;
import kyazuki.dataclass.Config;
import kyazuki.dataclass.Profiles;
import kyazuki.dataclass.Profiles.Profile;

public class ProfileAppenderController {
    @FXML
    public void initialize() {
        Service<Void> service = new Service<>() {
            @Override
            protected Task<Void> createTask() {
                return new Task<>() {
                    @Override
                    protected Void call() throws Exception {
                        Logger logger = App.getLogger();
                        Config config = App.getConfig();
                        // プロファイルを追加する
                        PROFILE: {
                            String os = System.getProperty("os.name");
                            if (!os.startsWith("Windows")) { // Windows以外のOSのとき
                                // プロファイルファイルのパスが不明なためスキップ
                                logger.warning("'" + os + "' is not supported. Skip adding profile.");
                                break PROFILE;
                            }
                            Path profilesPath = Path.of(System.getenv("APPDATA"), ".minecraft",
                                    "launcher_profiles.json");
                            if (!Files.exists(profilesPath)) { // プロファイルファイルが見つからないとき
                                logger.warning("No profile file found. Skip adding profile.");
                                break PROFILE;
                            }
                            Profiles profiles = null;
                            try (FileInputStream fis = new FileInputStream(profilesPath.toFile())) {
                                ObjectMapper json = new ObjectMapper();
                                json.registerModule(new JavaTimeModule());
                                profiles = json.readValue(fis, Profiles.class);
                            } catch (IOException e) {
                                logger.log(Level.WARNING, "Failed to load profile file. Skip adding profile.", e);
                                break PROFILE;
                            }
                            final String profileName = config.profile.name;
                            if (profiles.profiles.entrySet().stream()
                                    .anyMatch(e -> e.getValue().name.equals(profileName))) {
                                // 既に同じ名前のプロファイルが存在するとき
                                logger.warning("Profile already exists. Skip adding profile.");
                                break PROFILE;
                            }
                            String uuid = null;
                            while (true) {
                                uuid = UUID.randomUUID().toString().replace("-", "");
                                if (profiles.profiles.get(uuid) == null) {
                                    break;
                                }
                            }
                            Instant now = Instant.now();
                            Profile newProfile = new Profile();
                            newProfile.created = now;
                            newProfile.gameDir = Path.of(".").toAbsolutePath().getParent().toString();
                            newProfile.icon = config.profile.icon;
                            newProfile.javaArgs = config.profile.javaArgs;
                            newProfile.lastUsed = now;
                            newProfile.lastVersionId = config.profile.versionId;
                            newProfile.name = config.profile.name;
                            newProfile.resolution = null;
                            newProfile.type = "custom";
                            profiles.profiles.put(uuid, newProfile);
                            Path backupProfilesPath = profilesPath
                                    .resolveSibling(profilesPath.getFileName().toString() + ".bak");
                            for (int i = 1; Files.exists(backupProfilesPath); i++) {
                                backupProfilesPath = profilesPath
                                        .resolveSibling(profilesPath.getFileName().toString() + ".bak" + i);
                            }
                            if (!profilesPath.toFile().renameTo(backupProfilesPath.toFile())) {
                                // プロファイルファイルのバックアップ(リネーム)に失敗したとき
                                logger.warning("Failed to backup profile file. Skip adding profile.");
                                break PROFILE;
                            }
                            try (FileWriter fw = new FileWriter(profilesPath.toFile())) {
                                ObjectMapper json = new ObjectMapper();
                                JavaTimeModule module = new JavaTimeModule();
                                module.addSerializer(Instant.class, new Profiles.CustomInstantSerializer());
                                json.registerModule(module);
                                json.enable(SerializationFeature.INDENT_OUTPUT);
                                json.disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS);
                                fw.write(json.writeValueAsString(profiles));
                            } catch (IOException e) {
                                logger.log(Level.WARNING, "Failed to save profile file. Skip adding profile.", e);
                                break PROFILE;
                            }
                            logger.info("Added profile: " + profileName);
                        }
                        return null;
                    }
                };
            }
        };
        service.setOnSucceeded(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                Logger logger = App.getLogger();
                Config config = App.getConfig();
                boolean isOpenModLoader = config.modLoader.autoOpen;
                Alert alert = new Alert(AlertType.INFORMATION);
                alert.setTitle("完了");
                alert.setHeaderText(null);
                alert.setContentText(
                        (!isOpenModLoader) ? "インストールが完了しました。" : "インストールが完了しました。\nModLoaderのインストーラーが起動します。");
                alert.setOnCloseRequest(new EventHandler<DialogEvent>() {
                    @Override
                    public void handle(DialogEvent event) {
                        if (isOpenModLoader) {
                            try {
                                ProcessBuilder pb = new ProcessBuilder("java", "-jar", config.modLoader.getFileName());
                                pb.start();
                            } catch (IOException e) {
                                logger.log(Level.WARNING, "Failed to start ModLoader Installer.", e);
                            }
                        }
                        App.finish();
                    }
                });
                alert.show();
            }
        });
        service.setOnFailed(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                App.showAlertAndExit("プロファイルの追加に失敗しました。");
            }
        });
        service.start();
    }
}
