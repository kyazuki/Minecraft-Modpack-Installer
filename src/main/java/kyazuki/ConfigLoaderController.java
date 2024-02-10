package kyazuki;

import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.util.logging.Level;
import java.util.logging.Logger;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;

import javafx.concurrent.Service;
import javafx.concurrent.Task;
import javafx.concurrent.WorkerStateEvent;
import javafx.event.EventHandler;
import javafx.fxml.FXML;

public class ConfigLoaderController {
    @FXML
    public void initialize() {
        Service<Void> service = new Service<>() {
            @Override
            protected Task<Void> createTask() {
                return new Task<>() {
                    @Override
                    protected Void call() throws Exception {
                        Logger logger = App.getLogger();
                        // 設定ファイルを読み込む
                        File f = new File("config.yaml");
                        if (!f.exists()) { // 設定ファイルが見つからないとき
                            logger.severe("No config file found: " + f.getPath());
                            throw new IOException("No config file found: " + f.getPath());
                        }
                        Config config = null;
                        try (FileInputStream fis = new FileInputStream(f)) {
                            ObjectMapper yaml = new ObjectMapper(new YAMLFactory());
                            config = yaml.readValue(fis, Config.class);
                        } catch (IOException e) {
                            logger.log(Level.SEVERE, "Failed to load config file.", e);
                            throw e;
                        }
                        App.setConfig(config);
                        return null;
                    }
                };
            }
        };
        service.setOnSucceeded(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                try {
                    App.setRoot("modLoaderDownloader");
                } catch (IOException e) {
                    App.getLogger().log(Level.SEVERE, "Failed to transit modLoaderDownloader view.", e);
                    App.showAlertAndExit("ビュー遷移に失敗しました。");
                }
            }
        });
        service.setOnFailed(new EventHandler<WorkerStateEvent>() {
            @Override
            public void handle(WorkerStateEvent event) {
                App.showAlertAndExit("設定ファイルの読み込みに失敗しました。");
            }
        });
        service.start();
    }
}
